//! `KI-HIER-001` — a sheet pin and a hierarchical label disagree.
//!
//! **This rule implements no detector, and that is its whole design.** KiCad
//! 10's electrical rule check already answers this question under the key
//! `hier_label_mismatch`, and `spec/SPEC.md` §11.1 is explicit that KiCad's 47
//! checks are KiCad's: *"kicli's lint engine implements none of them."*
//! `research/style-rules.md` §4 lists this rule only so the published catalogue
//! is complete — *"kicli reports ERC's finding and gates on it."*
//!
//! So what is written below is the attribution and the gate. If a later edit
//! finds itself reading sheet pins and comparing them to hierarchical labels,
//! that edit is the failure this rule exists to prevent, and
//! `tests/the_hier_rule_adds_no_detector.rs` is the check that refuses it.
//!
//! # The tier is KiCad's own verdict, not kicli's assertion
//!
//! `lane-t5` measured 24 of 28 tiers in this catalogue as asserted rather than
//! argued. This is one of the four exceptions, and it is the strongest of them:
//! KiCad's **own default severity** for `hier_label_mismatch` is `ERROR`
//! (measured on 10.0.5 — `tests/fixtures/erc/hier/hier.erc.json` reports it as
//! `"severity": "error"` over a project whose `.kicad_pro` sets no severity at
//! all, and the key is absent from that report's `ignored_checks`). Tier 1 here
//! is kicli **agreeing with KiCad** rather than ruling over it, which no other
//! Tier 1 rule of this phase can say.
//!
//! What a mismatch does to a drawing is the north star's second sentence
//! directly: a sheet pin with no label behind it, or a label with no pin in
//! front of it, is a wire that the drawing shows going somewhere and the
//! netlist does not. A reader who follows the drawing is misled, and a reader
//! who follows the netlist cannot see the drawing's claim at all.
//!
//! # What this rule does when nobody ran the check, and why it is not enough
//!
//! [`RuleCheck::covers`](crate::lint::erc::RuleCheck::covers) answers `false` for two different situations on
//! purpose: the project turned the check off, and **nobody ran the check at
//! all**. A rule with its own geometry treats both the same way — it looks for
//! itself — and the conflation costs it nothing. **This rule has no own
//! geometry, so for it the two answers are a silence it cannot tell apart from
//! a clean drawing.**
//!
//! That silence is a correct thing for a *rule* to do. A rule records what is
//! wrong with a drawing, and "the drawing was never examined" is not a fact
//! about the drawing. Inventing a finding here would put a Tier 1 blocker on
//! every correct hierarchy on every machine with no KiCad install, and
//! `spec/SPEC.md` §6.1 already says what the absence of `kicad-cli` is: a
//! **structured error and exit 6**, not a finding.
//!
//! **But it means the gate verdict alone is not safe to read.** §11.2: `sch
//! score --gate` *"may require `kicad-cli`, because half of Tier 1 is
//! ERC-owned"* — and this rule is that half. A `--gate` run with no
//! `kicad-cli` that printed `gate: pass` would have told an agent its build is
//! fine on the strength of a check that never ran. The repair cannot live in
//! this file, because the finding stream carries no way to say it;
//! [`RuleCheck::has_run`](crate::lint::erc::RuleCheck::has_run) is public and the `sch score` surface must ask it
//! before it prints a verdict. `tests/lint_hier_label_mismatch.rs` holds that
//! hazard as a check over a drawing KiCad calls an error, so the silence is a
//! measured property rather than a surprise.
//!
//! # Saturation
//!
//! Declared [`Saturation::NEVER`], and for this rule the declaration is inert
//! twice over. [`crate::lint::gate::Gate::of`] never asks a Tier 1 rule what
//! share it covered — a blocking rule fails on its first finding, so the share
//! changes no verdict. And this rule's findings are **pairs**: one sheet pin
//! against one hierarchical label, counted across two sheets at once, where
//! [`crate::lint::gate::Counted`] offers the symbols or the wires of a single
//! sheet. Neither is this rule's denominator, so it declares nothing rather
//! than declaring something false.

use crate::lint::erc::delegate;
use crate::lint::gate::Saturation;
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};

/// The key KiCad's rule check reports this under.
///
/// `research/style-rules.md` §2.1 lists it among KiCad 10.0.5's 47 checks, and
/// §4 delegates this rule to it by name.
const CHECK: &str = "hier_label_mismatch";

/// A sheet pin and a hierarchical label disagree, as KiCad's ERC reported it.
pub struct HierLabelMismatch;

impl Rule for HierLabelMismatch {
    fn id(&self) -> RuleId {
        RuleId("KI-HIER-001")
    }

    /// Tier 1, which is KiCad's own default severity for this check.
    ///
    /// Tier 1 fails the gate on the first finding and moves no score: the
    /// weight stays at [`crate::lint::Penalty::ZERO`] by the trait's default,
    /// because a blocking finding is a verdict rather than a cost.
    fn tier(&self) -> Tier {
        Tier::One
    }

    /// Nothing countable. See the module documentation for why.
    fn saturation(&self) -> Saturation {
        Saturation::NEVER
    }

    /// Report what KiCad reported, and look for nothing.
    ///
    /// The one question is [`RuleCheck::covers`](crate::lint::erc::RuleCheck::covers): did KiCad actually run this
    /// check on this project. When it did, [`delegate`] records KiCad's own
    /// violations of it under this rule's code, with KiCad's message kept so a
    /// reader can tell which tool found it (§11.1's attribution half).
    ///
    /// When it did not — the project turned the check off, or nothing ran —
    /// there is no second branch, because there is no detector here to fall
    /// through to. The module documentation says what that silence costs and
    /// which surface owes the repair. The seam's shape writes a `return` here
    /// before its own geometry; this rule has none, so the `return` would be
    /// the needless one `clippy::needless_return` refuses.
    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        if drawing.rule_check().covers(CHECK) {
            delegate(CHECK, drawing, found);
        }
    }
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&HierLabelMismatch];
