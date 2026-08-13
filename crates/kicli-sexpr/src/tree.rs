//! A token-preserving tree, and the emitter that puts it back.
//!
//! Every atom remembers the bytes it came from. An atom the caller never edits
//! is written back from those bytes, so "kicli never rewrites a token it did not
//! modify" is a property of the data structure rather than a rule people have to
//! remember.

use crate::error::{SexprError, Span};
use crate::lexer::{TokenKind, lex};
use crate::number::parse_iu;
use crate::prettify::{FormatMode, detect_mode, prettify};
use crate::quote::unquote;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A handle to a node in a [`Doc`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Whether an atom was quoted in the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomKind {
    /// An unquoted run: a number, a keyword, or a symbol.
    Bare,
    /// A quoted string. The span includes both quote characters.
    Quoted,
}

/// One node of the tree.
#[derive(Clone, Debug)]
pub enum Node {
    /// A parenthesised list.
    List {
        /// The first child, when it is a bare atom. Memoised so a lookup by
        /// head token costs nothing.
        head: Option<NodeId>,
        /// The list's children, in source order.
        children: Vec<NodeId>,
        /// The bytes the list covers, parentheses included.
        span: Span,
    },
    /// A leaf token.
    Atom {
        /// Whether the source quoted it.
        kind: AtomKind,
        /// The bytes it came from.
        span: Span,
        /// Replacement text, once the caller has changed it.
        edited: Option<Box<str>>,
    },
    /// A `#` comment. KiCad drops these on save, and so does the emitter.
    Comment {
        /// The bytes the comment covers, excluding its newline.
        span: Span,
    },
}

/// A parsed file.
#[derive(Clone, Debug)]
pub struct Doc {
    source: Arc<str>,
    nodes: Vec<Node>,
    top: Vec<NodeId>,
    mode: FormatMode,
    canonical: bool,
}

impl Doc {
    /// Parse `source`.
    ///
    /// # Errors
    ///
    /// Returns a [`SexprError`] when the text is not a well-formed
    /// s-expression: an unterminated string, an unmatched parenthesis, or a
    /// list left open at the end of the file.
    pub fn parse(source: &str) -> Result<Self, SexprError> {
        let mut builder = Builder::default();
        for token in &lex(source)? {
            builder.absorb(*token)?;
        }
        let (nodes, top) = builder.finish()?;
        Ok(Self::assemble(source, nodes, top))
    }

    /// Work out the layout mode, then hand back the finished document.
    fn assemble(source: &str, nodes: Vec<Node>, top: Vec<NodeId>) -> Self {
        let source: Arc<str> = Arc::from(source);
        let detected = detect_mode(&source);
        let root_head = top
            .iter()
            .find_map(|&id| match &nodes[id.index()] {
                Node::List { head, .. } => head.map(|h| atom_source(&source, &nodes[h.index()])),
                _ => None,
            })
            .unwrap_or("");
        let mode = detected.unwrap_or_else(|| default_mode(root_head));

        Self {
            source,
            nodes,
            top,
            mode,
            canonical: detected.is_some(),
        }
    }

    /// The original text.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The layout mode the file is written in.
    #[must_use]
    pub fn mode(&self) -> FormatMode {
        self.mode
    }

    /// Was the file already laid out the way KiCad lays files out?
    ///
    /// A non-canonical file can still be written, but writing it reformats the
    /// whole file, which is what KiCad's next save would do anyway.
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        self.canonical
    }

    /// Does the file carry `#` comments?
    ///
    /// KiCad drops comments on save. A caller that cares must ask before
    /// writing.
    #[must_use]
    pub fn has_comments(&self) -> bool {
        self.nodes
            .iter()
            .any(|node| matches!(node, Node::Comment { .. }))
    }

    /// An atom's value as an internal unit, when it is one.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = kicli_sexpr::Doc::parse("(at 41.91 0)").expect("parses");
    /// let root = doc.root().expect("has a root");
    /// assert_eq!(doc.atom_as_iu(doc.children(root)[1]), Some(419_100));
    /// ```
    #[must_use]
    pub fn atom_as_iu(&self, id: NodeId) -> Option<i32> {
        parse_iu(self.atom_text(id)?)
    }

    /// An atom's text with its quotes and escapes resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = kicli_sexpr::Doc::parse(r#"(property "say \"hi\"")"#).expect("parses");
    /// let root = doc.root().expect("has a root");
    /// assert_eq!(doc.atom_as_str(doc.children(root)[1]).as_deref(), Some("say \"hi\""));
    /// ```
    #[must_use]
    pub fn atom_as_str(&self, id: NodeId) -> Option<String> {
        let text = self.atom_text(id)?;
        match self.nodes[id.index()] {
            Node::Atom {
                kind: AtomKind::Quoted,
                ..
            } => Some(unquote(text)),
            _ => Some(text.to_owned()),
        }
    }

    /// Every `(uuid ...)` in the file, mapped to the list that owns it.
    ///
    /// A UUID is the one handle every object carries, so this is how a caller
    /// finds an object it was told about earlier.
    ///
    /// # Examples
    ///
    /// ```
    /// let doc = kicli_sexpr::Doc::parse(
    ///     "(kicad_sch (junction (at 0 0) (uuid \"abc\")))",
    /// )
    /// .expect("parses");
    /// let index = doc.uuid_index();
    /// let owner = index.get("abc").copied().expect("the uuid is indexed");
    /// assert!(doc.head_is(owner, "junction"));
    /// ```
    #[must_use]
    pub fn uuid_index(&self) -> BTreeMap<String, NodeId> {
        let mut index = BTreeMap::new();
        self.index_uuids(&self.top, &mut index);
        index
    }

    fn index_uuids(&self, ids: &[NodeId], index: &mut BTreeMap<String, NodeId>) {
        for &id in ids {
            let Node::List { children, .. } = &self.nodes[id.index()] else {
                continue;
            };
            for &child in children {
                if self.head_is(child, "uuid")
                    && let Some(&atom) = self.children(child).get(1)
                    && let Some(value) = self.atom_as_str(atom)
                {
                    index.insert(value, id);
                }
            }
            self.index_uuids(children, index);
        }
    }

    /// Bare atoms that start with `#`, which cannot be written safely.
    ///
    /// `#` opens a comment when it is the first non-blank character on a line.
    /// Laying the file out can move such an atom to the start of a line, where
    /// reading it back yields a comment and swallows the rest of the line.
    /// KiCad never writes one, because it quotes every user string. A caller
    /// holding one must refuse to write rather than corrupt the file.
    #[must_use]
    pub fn unrepresentable_atoms(&self) -> Vec<NodeId> {
        self.node_ids()
            .filter(|&id| {
                matches!(
                    self.nodes[id.index()],
                    Node::Atom {
                        kind: AtomKind::Bare,
                        ..
                    }
                ) && self.atom_text(id).is_some_and(|t| t.starts_with('#'))
            })
            .collect()
    }

    /// The outermost list, which is the file's single s-expression.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.top
            .iter()
            .copied()
            .find(|&id| matches!(self.nodes[id.index()], Node::List { .. }))
    }

    /// Look at a node.
    #[must_use]
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.index()]
    }

    /// The children of a list, or an empty slice for anything else.
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        match &self.nodes[id.index()] {
            Node::List { children, .. } => children,
            _ => &[],
        }
    }

    /// The text of an atom: its edit when it has one, else its source bytes.
    #[must_use]
    pub fn atom_text(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.index()] {
            Node::Atom {
                edited: Some(t), ..
            } => Some(t),
            Node::Atom { span, .. } | Node::Comment { span } => Some(&self.source[span.clone()]),
            Node::List { .. } => None,
        }
    }

    /// The head token of a list, when it has a bare one.
    #[must_use]
    pub fn head(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.index()] {
            Node::List { head, .. } => head.and_then(|h| self.atom_text(h)),
            _ => None,
        }
    }

    /// Is this a list whose head is `token`?
    #[must_use]
    pub fn head_is(&self, id: NodeId, token: &str) -> bool {
        self.head(id) == Some(token)
    }

    /// Every node, in creation order.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        (0..self.nodes.len()).map(|i| NodeId(u32::try_from(i).unwrap_or(u32::MAX)))
    }

    /// How many tokens the file holds.
    ///
    /// A list counts as two tokens, for its two parentheses. This is the
    /// measure that makes "no token was lost" checkable.
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.nodes
            .iter()
            .map(|node| match node {
                Node::List { .. } => 2,
                _ => 1,
            })
            .sum()
    }

    /// Replace an atom's text.
    ///
    /// The caller supplies the exact token, quotes included for a quoted atom.
    ///
    /// # Panics
    ///
    /// Panics when `id` is not an atom.
    pub fn set_atom(&mut self, id: NodeId, text: &str) {
        match &mut self.nodes[id.index()] {
            Node::Atom { edited, .. } => *edited = Some(text.into()),
            _ => panic!("set_atom needs an atom"),
        }
    }

    /// Write the file back out.
    ///
    /// Unedited atoms come from their source bytes. Comments are dropped, which
    /// is what KiCad does.
    #[must_use]
    pub fn emit(&self) -> String {
        let mut flat = String::with_capacity(self.source.len());
        for (position, &id) in self
            .top
            .iter()
            .filter(|&&id| !matches!(self.nodes[id.index()], Node::Comment { .. }))
            .enumerate()
        {
            if position > 0 {
                flat.push(' ');
            }
            self.write_flat(id, &mut flat);
        }
        prettify(&flat, self.mode)
    }

    fn write_flat(&self, id: NodeId, out: &mut String) {
        match &self.nodes[id.index()] {
            Node::Comment { .. } => {}
            Node::Atom { .. } => out.push_str(self.atom_text(id).unwrap_or_default()),
            Node::List { children, .. } => {
                out.push('(');
                for (position, &child) in children
                    .iter()
                    .filter(|&&c| !matches!(self.nodes[c.index()], Node::Comment { .. }))
                    .enumerate()
                {
                    if position > 0 {
                        out.push(' ');
                    }
                    self.write_flat(child, out);
                }
                out.push(')');
            }
        }
    }

    /// Do two documents hold the same tokens in the same shape?
    ///
    /// Whitespace, spans and comments are ignored. This is the property that
    /// must hold for every file, including files KiCad did not write.
    #[must_use]
    pub fn structurally_eq(&self, other: &Self) -> bool {
        let mine: Vec<NodeId> = self.significant(&self.top);
        let theirs: Vec<NodeId> = other.significant(&other.top);
        mine.len() == theirs.len()
            && mine
                .iter()
                .zip(&theirs)
                .all(|(&a, &b)| self.same_node(a, other, b))
    }

    fn significant(&self, ids: &[NodeId]) -> Vec<NodeId> {
        ids.iter()
            .copied()
            .filter(|&id| !matches!(self.nodes[id.index()], Node::Comment { .. }))
            .collect()
    }

    fn same_node(&self, mine: NodeId, other: &Self, theirs: NodeId) -> bool {
        match (&self.nodes[mine.index()], &other.nodes[theirs.index()]) {
            (Node::Atom { kind: a, .. }, Node::Atom { kind: b, .. }) => {
                a == b && self.atom_text(mine) == other.atom_text(theirs)
            }
            (Node::List { children: a, .. }, Node::List { children: b, .. }) => {
                let a = self.significant(a);
                let b = other.significant(b);
                a.len() == b.len() && a.iter().zip(&b).all(|(&x, &y)| self.same_node(x, other, y))
            }
            _ => false,
        }
    }
}

/// Builds the arena as tokens arrive.
#[derive(Default)]
struct Builder {
    nodes: Vec<Node>,
    top: Vec<NodeId>,
    open: Vec<(usize, Vec<NodeId>)>,
}

impl Builder {
    /// Fold one token into the tree.
    fn absorb(&mut self, token: crate::lexer::Token) -> Result<(), SexprError> {
        match token.kind {
            TokenKind::LParen => {
                self.open.push((token.start, Vec::new()));
                return Ok(());
            }
            TokenKind::RParen => {
                let Some((start, children)) = self.open.pop() else {
                    return Err(SexprError::UnmatchedClose(token.start));
                };
                // A list's head is its first child when that child is bare.
                // Memoising it here makes a lookup by head token free.
                let head = children.first().copied().filter(|&id| {
                    matches!(
                        self.nodes[id.index()],
                        Node::Atom {
                            kind: AtomKind::Bare,
                            ..
                        }
                    )
                });
                let id = self.push(Node::List {
                    head,
                    children,
                    span: start..token.end,
                });
                self.attach(id);
            }
            TokenKind::Bare | TokenKind::Quoted => {
                let kind = if token.kind == TokenKind::Quoted {
                    AtomKind::Quoted
                } else {
                    AtomKind::Bare
                };
                let id = self.push(Node::Atom {
                    kind,
                    span: token.start..token.end,
                    edited: None,
                });
                self.attach(id);
            }
            TokenKind::Comment => {
                let id = self.push(Node::Comment {
                    span: token.start..token.end,
                });
                self.attach(id);
            }
        }
        Ok(())
    }

    /// Hand over the arena, once every list has closed.
    fn finish(self) -> Result<(Vec<Node>, Vec<NodeId>), SexprError> {
        if let Some((start, _)) = self.open.last() {
            return Err(SexprError::UnclosedList(*start));
        }
        if self
            .top
            .iter()
            .all(|&id| matches!(self.nodes[id.index()], Node::Comment { .. }))
        {
            return Err(SexprError::Empty);
        }
        Ok((self.nodes, self.top))
    }

    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId(u32::try_from(self.nodes.len()).unwrap_or(u32::MAX));
        self.nodes.push(node);
        id
    }

    fn attach(&mut self, id: NodeId) {
        if let Some((_, children)) = self.open.last_mut() {
            children.push(id);
        } else {
            self.top.push(id);
        }
    }
}

/// The mode to write a file in when its own layout does not say.
fn default_mode(root_head: &str) -> FormatMode {
    match root_head {
        "sym_lib_table" | "fp_lib_table" | "design_block_lib_table" => FormatMode::LibraryTable,
        _ => FormatMode::Normal,
    }
}

fn atom_source<'a>(source: &'a str, node: &Node) -> &'a str {
    match node {
        Node::Atom { span, .. } | Node::Comment { span } => &source[span.clone()],
        Node::List { .. } => "",
    }
}

/// How many lines differ between two texts.
///
/// Used to check that an edit stays where it was made. A change that touches
/// lines far from the edit is the failure mode that quietly ruins somebody's
/// git history.
#[must_use]
pub fn changed_line_count(before: &str, after: &str) -> usize {
    let before: Vec<&str> = before.lines().collect();
    let after: Vec<&str> = after.lines().collect();

    let leading = before
        .iter()
        .zip(&after)
        .take_while(|(a, b)| a == b)
        .count();
    let trailing = before[leading..]
        .iter()
        .rev()
        .zip(after[leading..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let before_changed = before.len().saturating_sub(leading + trailing);
    let after_changed = after.len().saturating_sub(leading + trailing);
    before_changed.max(after_changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_parses_and_comes_back() {
        let source = "(kicad_sch\n\t(version 20260306)\n)\n";
        let doc = Doc::parse(source).expect("parses");
        assert!(doc.is_canonical());
        assert_eq!(doc.emit(), source);
    }

    #[test]
    fn the_head_token_is_memoised() {
        let doc = Doc::parse("(symbol (at 1 2))").expect("parses");
        let root = doc.root().expect("has a root");
        assert!(doc.head_is(root, "symbol"));
        assert!(!doc.head_is(root, "wire"));
    }

    #[test]
    fn an_edit_shows_in_the_output() {
        let mut doc = Doc::parse("(a (b 1))").expect("parses");
        let root = doc.root().expect("has a root");
        let inner = doc.children(root)[1];
        let value = doc.children(inner)[1];
        doc.set_atom(value, "2");
        assert!(doc.emit().contains("(b 2)"));
    }

    #[test]
    fn unmatched_parentheses_are_errors() {
        assert!(matches!(Doc::parse("(a"), Err(SexprError::UnclosedList(0))));
        assert!(matches!(
            Doc::parse("(a))"),
            Err(SexprError::UnmatchedClose(3))
        ));
    }

    #[test]
    fn comments_are_dropped_on_write_as_kicad_drops_them() {
        let doc = Doc::parse("# note\n(a b)\n").expect("parses");
        assert!(doc.has_comments());
        assert_eq!(doc.emit(), "(a b)\n");
    }

    #[test]
    fn a_bare_hash_atom_is_reported_as_unrepresentable() {
        let doc = Doc::parse("(a #PWR01)").expect("parses");
        assert_eq!(doc.unrepresentable_atoms().len(), 1);

        let quoted = Doc::parse("(a \"#PWR01\")").expect("parses");
        assert!(quoted.unrepresentable_atoms().is_empty());
    }

    #[test]
    fn structural_equality_ignores_layout() {
        let one = Doc::parse("(a (b c))").expect("parses");
        let two = Doc::parse("(a\n\t(b   c)\n)").expect("parses");
        assert!(one.structurally_eq(&two));

        let different = Doc::parse("(a (b d))").expect("parses");
        assert!(!one.structurally_eq(&different));
    }

    #[test]
    fn changed_lines_are_counted_from_both_ends() {
        assert_eq!(changed_line_count("a\nb\nc", "a\nB\nc"), 1);
        assert_eq!(changed_line_count("a\nb\nc", "a\nb\nc"), 0);
        assert_eq!(changed_line_count("a\nb\nc", "a\nX\nY\nc"), 2);
    }
}
