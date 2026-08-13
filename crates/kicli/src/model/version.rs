//! What a file's version stamp changes about the meaning of its tokens.
//!
//! KiCad's schematic format is stamped with a date, not a version number, and
//! two of those stamps changed what existing tokens mean rather than adding new
//! ones. Reading a file without checking its stamp silently renames pins.

/// A schematic format stamp, as it appears in `(version ...)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormatVersion(u32);

/// The newest stamp kicli was built against, written by KiCad 10.0.5.
pub const MAX_SCHEMATIC_VERSION: FormatVersion = FormatVersion(20_260_306);

/// The stamp at which `~` stopped meaning "no text".
const TILDE_STOPPED_MEANING_EMPTY: u32 = 20_250_318;

/// The stamp at which `hide` moved out of `effects`.
const HIDE_LEFT_EFFECTS: u32 = 20_251_028;

impl FormatVersion {
    /// Wrap a raw stamp.
    #[must_use]
    pub const fn new(stamp: u32) -> Self {
        Self(stamp)
    }

    /// The raw stamp.
    #[must_use]
    pub const fn stamp(self) -> u32 {
        self.0
    }

    /// Does `~` mean "no text" in this file?
    ///
    /// Before this changed, a pin name or number of `~` meant the empty string.
    /// After it, the empty string means that and `~` is a literal tilde. A tool
    /// that ignores the stamp turns every unnamed pin into one named `~`, or
    /// the reverse.
    #[must_use]
    pub const fn tilde_means_empty(self) -> bool {
        self.0 < TILDE_STOPPED_MEANING_EMPTY
    }

    /// Does `hide` sit inside `effects` in this file?
    ///
    /// It used to be a child of `effects`. It is now a child of the property,
    /// alongside `show_name` and `do_not_autoplace`.
    #[must_use]
    pub const fn hide_lives_in_effects(self) -> bool {
        self.0 < HIDE_LEFT_EFFECTS
    }

    /// Is this stamp newer than kicli understands?
    #[must_use]
    pub const fn is_newer_than_known(self) -> bool {
        self.0 > MAX_SCHEMATIC_VERSION.0
    }
}

/// Resolve the text of a pin name or number, given the file's stamp.
///
/// # Examples
///
/// ```
/// use kicli::model::{FormatVersion, pin_text};
/// let v9 = FormatVersion::new(20_250_114);
/// let v10 = FormatVersion::new(20_260_306);
/// assert_eq!(pin_text("~", v9), "");
/// assert_eq!(pin_text("~", v10), "~");
/// ```
#[must_use]
pub fn pin_text(raw: &str, version: FormatVersion) -> &str {
    if version.tilde_means_empty() && raw == "~" {
        ""
    } else {
        raw
    }
}

/// Where `hide` goes when kicli writes a property.
///
/// The two orderings are not interchangeable. A property on a placed symbol
/// writes `hide` before `show_name`; the same property inside `lib_symbols`
/// writes it last. An emitter that uses one ordering everywhere produces a file
/// that differs from KiCad's byte for byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyOrder {
    /// A property on a placed item: `at`, `hide`, `show_name`,
    /// `do_not_autoplace`, `effects`.
    Instance,
    /// A property inside `lib_symbols`: `at`, `show_name`, `do_not_autoplace`,
    /// `hide`, `effects`.
    Library,
}

impl PropertyOrder {
    /// The order the child tokens are written in.
    #[must_use]
    pub const fn tokens(self) -> &'static [&'static str] {
        match self {
            Self::Instance => &["at", "hide", "show_name", "do_not_autoplace", "effects"],
            Self::Library => &["at", "show_name", "do_not_autoplace", "hide", "effects"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tilde_rule_turns_over_at_its_stamp() {
        assert!(FormatVersion::new(20_250_114).tilde_means_empty());
        assert!(FormatVersion::new(TILDE_STOPPED_MEANING_EMPTY - 1).tilde_means_empty());
        assert!(!FormatVersion::new(TILDE_STOPPED_MEANING_EMPTY).tilde_means_empty());
        assert!(!FormatVersion::new(20_260_306).tilde_means_empty());
    }

    #[test]
    fn the_hide_rule_turns_over_at_its_stamp() {
        assert!(FormatVersion::new(20_250_114).hide_lives_in_effects());
        assert!(!FormatVersion::new(20_260_306).hide_lives_in_effects());
    }

    #[test]
    fn the_two_property_orderings_are_not_the_same() {
        let instance = PropertyOrder::Instance.tokens();
        let library = PropertyOrder::Library.tokens();
        assert_ne!(instance, library);

        let hide_at = |order: &[&str]| order.iter().position(|t| *t == "hide").expect("has hide");
        assert!(
            hide_at(instance) < hide_at(library),
            "an instance property writes hide before a library property does"
        );
    }

    #[test]
    fn a_newer_stamp_is_recognised_as_newer() {
        assert!(FormatVersion::new(20_260_803).is_newer_than_known());
        assert!(!FormatVersion::new(20_260_306).is_newer_than_known());
    }
}
