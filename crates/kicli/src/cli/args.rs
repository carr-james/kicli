//! The `kicli <noun> <verb> [flags] [args]` surface.
//!
//! One type per noun, one variant per verb. The global flags are declared once
//! and accepted before or after the verb, so an agent does not have to remember
//! where a flag goes.
//!
//! Positions, sizes and pins arrive as text and leave as typed values, so a
//! malformed one is a usage error the argument parser reports rather than a
//! surprise deeper in. Millimetres are the unit at this boundary and nowhere
//! else, because that is the unit the views print.

use crate::edit::field::{Horizontal, Vertical};
use crate::edit::label::PortShape;
use crate::geometry::{Angle, Iu, Point, Size};
use crate::model::{LabelKind, Mirror, Refdes};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::str::FromStr;

/// The whole command line.
#[derive(Debug, Parser)]
#[command(
    name = "kicli",
    version,
    about = "Read and edit KiCad 10 schematics from the command line.",
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The flags every command accepts.
    #[command(flatten)]
    pub global: Global,

    /// The noun, and its verb.
    #[command(subcommand)]
    pub command: Command,
}

/// The flags every command accepts.
#[derive(Args, Clone, Debug)]
pub struct Global {
    /// Print results as text or as JSON.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,

    /// The project directory. The default is the working directory.
    #[arg(long, short = 'p', global = true, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// One sheet path, to read or edit instead of the root sheet.
    #[arg(long, global = true, value_name = "PATH")]
    pub sheet: Option<String>,

    /// Print results only. Suppress progress notes.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Write a file that carries `#` comments, losing them as KiCad would.
    #[arg(long, global = true)]
    pub allow_comment_loss: bool,

    // A variant selects one set of field overrides. kicli keeps variant data
    // through a round trip and does not act on it, so the flag is accepted and
    // hidden rather than advertised.
    /// A design variant. Accepted, and without effect in this version.
    #[arg(long, global = true, hide = true, value_name = "NAME")]
    pub variant: Option<String>,
}

/// How a result is printed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Terse lines, for a person and for an agent's context budget.
    #[default]
    Text,
    /// One JSON object, carrying the same content.
    Json,
}

/// The nouns.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read a whole project.
    Project {
        /// What to report about it.
        #[command(subcommand)]
        verb: ProjectVerb,
    },
    /// Read the schematics.
    Sch {
        /// What to do with them.
        #[command(subcommand)]
        verb: SchVerb,
    },
    /// Place and move symbols.
    Sym {
        /// What to do with one.
        #[command(subcommand)]
        verb: SymVerb,
    },
    /// Move and hide the text a symbol, sheet or label owns.
    Field {
        /// What to do with one.
        #[command(subcommand)]
        verb: FieldVerb,
    },
    /// Draw free text and text boxes.
    Text {
        /// What to do with one.
        #[command(subcommand)]
        verb: TextVerb,
    },
    /// Name nets with local, global and hierarchical labels.
    Label {
        /// What to do with one.
        #[command(subcommand)]
        verb: LabelVerb,
    },
    /// Join crossing wires.
    Junction {
        /// What to do with one.
        #[command(subcommand)]
        verb: JunctionVerb,
    },
    /// Mark a pin as deliberately unconnected.
    Noconnect {
        /// What to do with one.
        #[command(subcommand)]
        verb: NoconnectVerb,
    },
    /// Work on a whole net.
    Net {
        /// What to do with one.
        #[command(subcommand)]
        verb: NetVerb,
    },
}

/// The verbs of the `sch` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum SchVerb {
    /// Print a compact view of the drawing.
    ///
    /// The delta view answers one question: what has touched this file since
    /// kicli last wrote it? It is empty right after a mutation, by design.
    /// That mutation's own result already reported what it changed. Keep that
    /// result. Nothing derives it again.
    View {
        /// Which view to print.
        #[arg(long, value_enum, default_value_t = ViewName::Connectivity)]
        view: ViewName,

        /// List power symbols, which are otherwise left out as noise.
        #[arg(long)]
        include_power: bool,

        /// Add the first eight characters of each object's identifier.
        #[arg(long)]
        uuids: bool,

        /// Report the size of the view in bytes.
        #[arg(long)]
        stats: bool,

        /// The saved state the delta compares against.
        ///
        /// The default is the state every command that writes leaves behind.
        #[arg(long, value_name = "NAME")]
        against: Option<String>,
    },
}

/// The views `sch view` can print.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ViewName {
    /// What is joined to what.
    #[default]
    Connectivity,
    /// Where things are drawn.
    Layout,
    /// What has touched the file since kicli last wrote it.
    Delta,
}

/// The verbs of the `project` noun.
#[derive(Clone, Copy, Debug, Subcommand)]
pub enum ProjectVerb {
    /// Report what the project is.
    Info,
    /// Report what is wrong with the project.
    Check,
}

/// The verbs of the `sym` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum SymVerb {
    /// Place a symbol, with its definition and its instance data.
    Place(PlaceArgs),

    /// Move a symbol, and carry its fields with it.
    Move(MoveArgs),

    /// Turn a symbol to an absolute angle.
    Rotate {
        /// The reference designator, or the identifier, of the symbol.
        target: String,

        /// The angle to turn it to.
        #[arg(long, value_enum)]
        to: RightAngle,

        /// Leave the fields where they are instead of carrying them.
        #[arg(long)]
        keep_field_positions: bool,
    },

    /// Mirror a symbol about an axis through its own anchor.
    Mirror {
        /// The reference designator, or the identifier, of the symbol.
        target: String,

        /// The axis to reflect about.
        #[arg(long, value_enum)]
        axis: Axis,

        /// Leave the fields where they are instead of carrying them.
        #[arg(long)]
        keep_field_positions: bool,
    },

    /// Delete a symbol, its instance data and its unused definition.
    Delete {
        /// The reference designator, or the identifier, of the symbol.
        target: String,
    },

    /// Set the text of one field of a symbol.
    SetField {
        /// The reference designator, or the identifier, of the symbol.
        target: String,

        /// The field name, such as `Reference` or `Value`.
        #[arg(long, value_name = "FIELD")]
        name: String,

        /// The text to write.
        #[arg(long, value_name = "TEXT")]
        value: String,
    },
}

/// What `sym place` places, and where.
#[derive(Args, Clone, Debug)]
pub struct PlaceArgs {
    /// The library identifier to record, such as `Device:R`.
    #[arg(long, value_name = "ID")]
    pub lib_id: String,

    /// A `.kicad_sym` file holding the definition.
    ///
    /// Without it, the definition must already be embedded in this project.
    #[arg(long, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// Where the anchor goes, in millimetres.
    #[arg(long, value_name = "X,Y")]
    pub at: PointArg,

    /// The reference designator the new placement carries.
    #[arg(long, value_name = "REF")]
    pub reference: String,

    /// The value to write, when it is not the library's own.
    #[arg(long, value_name = "TEXT")]
    pub value: Option<String>,

    /// The angle to place it at.
    #[arg(long, value_enum, default_value_t = RightAngle::Zero)]
    pub angle: RightAngle,

    /// The axis to mirror it about, applied after the angle.
    #[arg(long, value_enum)]
    pub mirror: Option<Axis>,

    /// Which unit of a multi-unit part to draw.
    #[arg(long, default_value_t = 1)]
    pub unit: u32,

    /// Which body style to draw: 1 is normal, 2 is the De Morgan form.
    #[arg(long, default_value_t = 1)]
    pub body_style: u32,

    /// Place the anchor exactly where it was asked for, off the grid.
    #[arg(long)]
    pub off_grid: bool,
}

/// Where `sym move` puts a symbol.
#[derive(Args, Clone, Debug)]
#[command(group = clap::ArgGroup::new("motion").required(true).args(["to", "by"]))]
pub struct MoveArgs {
    /// The reference designator, or the identifier, of the symbol.
    pub target: String,

    /// The position to move it to, in millimetres.
    #[arg(long, value_name = "X,Y")]
    pub to: Option<PointArg>,

    /// The offset to move it by, in millimetres.
    #[arg(long, value_name = "DX,DY")]
    pub by: Option<PointArg>,

    /// Place the anchor exactly where it was asked for, off the grid.
    #[arg(long)]
    pub off_grid: bool,

    /// Leave the fields where they are instead of carrying them.
    #[arg(long)]
    pub keep_field_positions: bool,
}

/// The verbs of the `field` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum FieldVerb {
    /// Put a field at a position.
    Move {
        /// The reference designator, or the identifier, of the owner.
        owner: String,

        /// The field name.
        #[arg(long, value_name = "FIELD")]
        name: String,

        /// The position to move it to, in millimetres.
        #[arg(long, value_name = "X,Y")]
        to: PointArg,
    },

    /// Turn a field to an angle.
    Rotate {
        /// The reference designator, or the identifier, of the owner.
        owner: String,

        /// The field name.
        #[arg(long, value_name = "FIELD")]
        name: String,

        /// The angle to turn it to.
        #[arg(long, value_enum)]
        to: RightAngle,
    },

    /// Set which part of a field's text sits at its position.
    #[command(group = clap::ArgGroup::new("alignment")
        .required(true)
        .multiple(true)
        .args(["horizontal", "vertical"]))]
    Justify {
        /// The reference designator, or the identifier, of the owner.
        owner: String,

        /// The field name.
        #[arg(long, value_name = "FIELD")]
        name: String,

        /// Where the text sits from left to right.
        #[arg(long, value_enum)]
        horizontal: Option<HorizontalArg>,

        /// Where the text sits from top to bottom.
        #[arg(long, value_enum)]
        vertical: Option<VerticalArg>,
    },

    /// Draw a field again.
    Show {
        /// The reference designator, or the identifier, of the owner.
        owner: String,

        /// The field name.
        #[arg(long, value_name = "FIELD")]
        name: String,
    },

    /// Stop drawing a field.
    Hide {
        /// The reference designator, or the identifier, of the owner.
        owner: String,

        /// The field name.
        #[arg(long, value_name = "FIELD")]
        name: String,
    },
}

/// The verbs of the `text` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum TextVerb {
    /// Add free text, or a text box, to a sheet.
    Add {
        /// The text to draw.
        #[arg(long, value_name = "TEXT")]
        text: String,

        /// Where the text is drawn, in millimetres.
        #[arg(long, value_name = "X,Y")]
        at: PointArg,

        /// The text angle.
        #[arg(long, value_enum, default_value_t = RightAngle::Zero)]
        angle: RightAngle,

        /// The width and height of a text box, in millimetres.
        #[arg(long, value_name = "WxH")]
        size: Option<SizeArg>,
    },

    /// Move text to a position.
    Move {
        /// The identifier of the text.
        target: String,

        /// The position to move it to, in millimetres.
        #[arg(long, value_name = "X,Y")]
        to: PointArg,
    },

    /// Change what the text says, or how large its box is.
    ///
    /// One command is one write, so the two are separate runs.
    #[command(group = clap::ArgGroup::new("content")
        .required(true)
        .args(["text", "size"]))]
    Edit {
        /// The identifier of the text.
        target: String,

        /// The text to draw instead.
        #[arg(long, value_name = "TEXT")]
        text: Option<String>,

        /// The width and height to give a text box, in millimetres.
        #[arg(long, value_name = "WxH")]
        size: Option<SizeArg>,
    },

    /// Take text off a sheet.
    Delete {
        /// The identifier of the text.
        target: String,
    },
}

/// The verbs of the `label` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum LabelVerb {
    /// Add a label to a sheet.
    Add {
        /// The net name the label carries.
        #[arg(long, value_name = "NAME")]
        text: String,

        /// Where the anchor goes, in millimetres.
        #[arg(long, value_name = "X,Y")]
        at: PointArg,

        /// Which kind of label to make.
        #[arg(long, value_enum, default_value_t = LabelKindArg::Local)]
        kind: LabelKindArg,

        /// The text angle.
        #[arg(long, value_enum, default_value_t = RightAngle::Zero)]
        angle: RightAngle,

        /// The direction a global or hierarchical label faces.
        #[arg(long, value_enum, default_value_t = ShapeArg::Passive)]
        shape: ShapeArg,
    },

    /// Move a label's anchor to a position.
    Move {
        /// The identifier of the label.
        target: String,

        /// The position to move it to, in millimetres.
        #[arg(long, value_name = "X,Y")]
        to: PointArg,
    },

    /// Take a label off a sheet.
    Delete {
        /// The identifier of the label.
        target: String,
    },
}

/// The verbs of the `junction` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum JunctionVerb {
    /// Add a junction at a point.
    #[command(group = clap::ArgGroup::new("where").required(true).args(["at", "pin"]))]
    Add {
        /// Where the junction goes, in millimetres.
        #[arg(long, value_name = "X,Y")]
        at: Option<PointArg>,

        /// The pin whose connection point the junction goes on.
        #[arg(long, value_name = "REF.PIN")]
        pin: Option<PinArg>,
    },

    /// Delete the junction at a point.
    #[command(group = clap::ArgGroup::new("where").required(true).args(["at", "pin"]))]
    Delete {
        /// Where the junction is, in millimetres.
        #[arg(long, value_name = "X,Y")]
        at: Option<PointArg>,

        /// The pin whose connection point the junction is on.
        #[arg(long, value_name = "REF.PIN")]
        pin: Option<PinArg>,
    },
}

/// The verbs of the `noconnect` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum NoconnectVerb {
    /// Add a no-connect to a pin.
    Add {
        /// The pin to mark.
        #[arg(long, value_name = "REF.PIN")]
        pin: PinArg,
    },

    /// Delete the no-connect on a pin.
    Delete {
        /// The pin to unmark.
        #[arg(long, value_name = "REF.PIN")]
        pin: PinArg,
    },
}

/// The verbs of the `net` noun.
#[derive(Clone, Debug, Subcommand)]
pub enum NetVerb {
    /// Rename a net, everywhere it reaches.
    Rename {
        /// The name kicli shows the net under.
        from: String,

        /// The name to give it.
        #[arg(long, value_name = "NAME")]
        to: String,
    },
}

/// The four angles a schematic object can take.
///
/// The set is closed, so an angle KiCad refuses is a usage error the argument
/// parser reports rather than a refusal from deeper in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RightAngle {
    /// No rotation.
    #[value(name = "0")]
    Zero,
    /// A quarter turn.
    #[value(name = "90")]
    Ninety,
    /// A half turn.
    #[value(name = "180")]
    OneEighty,
    /// Three quarters of a turn.
    #[value(name = "270")]
    TwoSeventy,
}

impl RightAngle {
    /// The angle itself.
    #[must_use]
    pub const fn angle(self) -> Angle {
        match self {
            Self::Zero => Angle(0),
            Self::Ninety => Angle(90),
            Self::OneEighty => Angle(180),
            Self::TwoSeventy => Angle(270),
        }
    }
}

/// An axis to mirror about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Axis {
    /// The horizontal line through the anchor, which the file writes as `x`.
    X,
    /// The vertical line through the anchor, which the file writes as `y`.
    Y,
}

impl Axis {
    /// The axis itself.
    #[must_use]
    pub const fn mirror(self) -> Mirror {
        match self {
            Self::X => Mirror::X,
            Self::Y => Mirror::Y,
        }
    }
}

/// Where a field's text sits from left to right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum HorizontalArg {
    /// The text starts at the position.
    Left,
    /// The text is centred on the position.
    Center,
    /// The text ends at the position.
    Right,
}

impl HorizontalArg {
    /// The alignment itself.
    #[must_use]
    pub const fn alignment(self) -> Horizontal {
        match self {
            Self::Left => Horizontal::Left,
            Self::Center => Horizontal::Center,
            Self::Right => Horizontal::Right,
        }
    }
}

/// Where a field's text sits from top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum VerticalArg {
    /// The text hangs below the position.
    Top,
    /// The text is centred on the position.
    Center,
    /// The text stands above the position.
    Bottom,
}

impl VerticalArg {
    /// The alignment itself.
    #[must_use]
    pub const fn alignment(self) -> Vertical {
        match self {
            Self::Top => Vertical::Top,
            Self::Center => Vertical::Center,
            Self::Bottom => Vertical::Bottom,
        }
    }
}

/// The kinds of label a caller can add.
///
/// A netclass flag carries a netclass name and not a net name, so it is not one
/// of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LabelKindArg {
    /// Names a net on the sheet it is drawn on.
    Local,
    /// Names a net across every sheet of the design.
    Global,
    /// The child half of a sheet port.
    Hierarchical,
}

impl LabelKindArg {
    /// The kind itself.
    #[must_use]
    pub const fn kind(self) -> LabelKind {
        match self {
            Self::Local => LabelKind::Local,
            Self::Global => LabelKind::Global,
            Self::Hierarchical => LabelKind::Hierarchical,
        }
    }
}

/// The direction a port faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ShapeArg {
    /// The port takes a signal in.
    Input,
    /// The port sends a signal out.
    Output,
    /// The port does both.
    Bidirectional,
    /// The port is driven by more than one source.
    TriState,
    /// The port has no direction.
    Passive,
}

impl ShapeArg {
    /// The direction itself.
    #[must_use]
    pub const fn shape(self) -> PortShape {
        match self {
            Self::Input => PortShape::Input,
            Self::Output => PortShape::Output,
            Self::Bidirectional => PortShape::Bidirectional,
            Self::TriState => PortShape::TriState,
            Self::Passive => PortShape::Passive,
        }
    }
}

/// A position in millimetres, written `X,Y`.
///
/// # Examples
///
/// ```
/// use kicli::cli::PointArg;
/// use kicli::geometry::{Iu, Point};
///
/// let point: PointArg = "50.8,-88.9".parse().expect("two readings");
/// assert_eq!(point.point(), Point { x: Iu(508_000), y: Iu(-889_000) });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointArg(Point);

impl PointArg {
    /// The position itself.
    #[must_use]
    pub const fn point(self) -> Point {
        self.0
    }
}

impl FromStr for PointArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (x, y) = split_pair(text, ',')?;
        Ok(Self(Point {
            x: millimetres(x)?,
            y: millimetres(y)?,
        }))
    }
}

/// A width and a height in millimetres, written `WxH`.
///
/// # Examples
///
/// ```
/// use kicli::cli::SizeArg;
/// use kicli::geometry::Size;
///
/// let size: SizeArg = "25.4x12.7".parse().expect("two readings");
/// assert_eq!(size.size(), Size::new(254_000, 127_000));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeArg(Size);

impl SizeArg {
    /// The size itself.
    #[must_use]
    pub const fn size(self) -> Size {
        self.0
    }
}

impl FromStr for SizeArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (width, height) = split_pair(text, 'x')?;
        Ok(Self(Size {
            x: millimetres(width)?,
            y: millimetres(height)?,
        }))
    }
}

/// One pin of one placed symbol, written `REF.PIN`.
///
/// The pin number is everything after the last stop, because a reference
/// designator never holds one and a pin number can.
///
/// # Examples
///
/// ```
/// use kicli::cli::PinArg;
///
/// let pin: PinArg = "R11.2".parse().expect("a reference and a number");
/// assert_eq!(pin.address().to_string(), "R11.2");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinArg {
    /// The reference designator.
    reference: Refdes,
    /// The pin number, as text.
    number: String,
}

impl PinArg {
    /// The pin, as the editing commands name it.
    #[must_use]
    pub fn address(&self) -> crate::edit::mark::PinAddress {
        crate::edit::mark::PinAddress::new(self.reference.clone(), &self.number)
    }
}

impl FromStr for PinArg {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (reference, number) = text
            .rsplit_once('.')
            .ok_or_else(|| format!("{text:?} is not REF.PIN, such as R11.2"))?;
        if reference.is_empty() || number.is_empty() {
            return Err(format!("{text:?} is not REF.PIN, such as R11.2"));
        }
        Ok(Self {
            reference: Refdes(reference.to_owned()),
            number: number.to_owned(),
        })
    }
}

/// Split `text` on one separator, and say so when it holds none.
fn split_pair(text: &str, separator: char) -> Result<(&str, &str), String> {
    text.split_once(separator)
        .ok_or_else(|| format!("{text:?} needs two readings separated by {separator:?}"))
}

/// Read one millimetre reading into internal units.
fn millimetres(text: &str) -> Result<Iu, String> {
    Iu::from_millimetres_text(text.trim())
        .ok_or_else(|| format!("{text:?} is not a millimetre reading kicli can represent"))
}

#[cfg(test)]
mod tests {
    use super::{Cli, PinArg, PointArg, SizeArg};
    use clap::{CommandFactory, Parser};

    #[test]
    fn the_surface_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn a_position_reads_as_millimetres() {
        let point: PointArg = "50.8,88.9".parse().expect("two readings");
        assert_eq!(point.point().to_string(), "50.8,88.9");
        assert!(
            "50.8".parse::<PointArg>().is_err(),
            "one reading is not two"
        );
        assert!("a,b".parse::<PointArg>().is_err(), "letters are not a size");
    }

    #[test]
    fn a_size_reads_as_millimetres() {
        let size: SizeArg = "25.4x12.7".parse().expect("two readings");
        assert_eq!(size.size().to_string(), "25.4x12.7");
        assert!("25.4".parse::<SizeArg>().is_err());
    }

    #[test]
    fn a_pin_splits_at_the_last_stop() {
        let pin: PinArg = "U1.A1".parse().expect("a reference and a number");
        assert_eq!(pin.address().to_string(), "U1.A1");
        assert!("R11".parse::<PinArg>().is_err(), "a pin needs a number");
        assert!("R11.".parse::<PinArg>().is_err(), "and the number is text");
    }

    #[test]
    fn a_move_needs_exactly_one_motion() {
        assert!(
            Cli::try_parse_from(["kicli", "sym", "move", "R1"]).is_err(),
            "a move with no motion is a usage error"
        );
        assert!(
            Cli::try_parse_from(["kicli", "sym", "move", "R1", "--to", "0,0", "--by", "0,0"])
                .is_err(),
            "a move cannot be both absolute and relative"
        );
        assert!(Cli::try_parse_from(["kicli", "sym", "move", "R1", "--to", "0,0"]).is_ok());
    }

    #[test]
    fn an_angle_off_the_quarter_turn_is_a_usage_error() {
        assert!(Cli::try_parse_from(["kicli", "sym", "rotate", "R1", "--to", "90"]).is_ok());
        assert!(Cli::try_parse_from(["kicli", "sym", "rotate", "R1", "--to", "45"]).is_err());
    }
}
