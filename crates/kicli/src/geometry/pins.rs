//! Where a placed symbol's pins actually are.
//!
//! ```text
//! abs_pin = symbol.at + M . (lib_pin.x, -lib_pin.y)
//! ```
//!
//! `M` is the orientation matrix and the negation is the library's Y-up to
//! schematic Y-down flip, which the library reader has already applied. Every
//! step is integer arithmetic, so a pin either lands on the grid or does not.
//!
//! Ported from `eeschema/sch_pin.cpp` and `eeschema/sch_symbol.cpp` at tag
//! 10.0.5.

use crate::geometry::{Angle, Point, Transform};
use crate::model::items::{Symbol, Uuid};
use crate::model::library::{LibraryPin, LibrarySymbol};

/// A pin of a placed symbol, with its position resolved.
#[derive(Clone, Debug)]
pub struct ResolvedPin {
    /// The pin number, as text.
    pub number: String,
    /// The pin name.
    pub name: String,
    /// Where the pin connects, in schematic coordinates.
    pub position: Point,
    /// Which way the pin body runs from its connection point, in schematic
    /// sense: 0 right, 90 down, 180 left, 270 up.
    pub direction: Angle,
    /// The electrical type, such as `passive` or `power_in`.
    pub electrical: String,
    /// Is the pin hidden? A hidden power pin still connects.
    pub hidden: bool,
    /// The pin instance's own identifier, when the placement records one.
    ///
    /// KiCad's rule check reports this, so it joins a finding to a pin without
    /// going through a refdes.
    pub uuid: Option<Uuid>,
}

/// Resolve every pin a placement draws.
///
/// The pins come from the units the placement selects, which are its own unit
/// and body style plus the common ones.
#[must_use]
pub fn resolve_pins(symbol: &Symbol, definition: &LibrarySymbol) -> Vec<ResolvedPin> {
    let transform = Transform::from_file(symbol.angle, symbol.mirror);
    definition
        .pins_for(symbol.unit, symbol.body_style)
        .map(|pin| resolve_pin(symbol, pin, transform))
        .collect()
}

fn resolve_pin(symbol: &Symbol, pin: &LibraryPin, transform: Transform) -> ResolvedPin {
    let offset = transform.apply(pin.at);
    let uuid = symbol
        .pins
        .iter()
        .find(|instance| instance.number == pin.number)
        .map(|instance| instance.uuid.clone());
    ResolvedPin {
        number: pin.number.clone(),
        name: pin.name.clone(),
        position: Point {
            x: crate::geometry::Iu(symbol.at.x.0 + offset.x.0),
            y: crate::geometry::Iu(symbol.at.y.0 + offset.y.0),
        },
        direction: direction_of(pin.angle, transform),
        electrical: pin.electrical.clone(),
        hidden: pin.hidden,
        uuid,
    }
}

/// Which way a pin runs once its symbol is placed.
///
/// The library angle is in library sense, where Y is up, so the unit vector is
/// flipped before the matrix is applied, exactly as the position is.
fn direction_of(angle: Angle, transform: Transform) -> Angle {
    let (dx, dy) = match angle.0.rem_euclid(360) {
        90 => (0, 1),
        180 => (-1, 0),
        270 => (0, -1),
        _ => (1, 0),
    };
    // Library Y-up to schematic Y-down, then the placement's orientation.
    let flipped = Point::new(dx, -dy);
    let mapped = transform.apply(flipped);
    match (mapped.x.0.signum(), mapped.y.0.signum()) {
        (1, 0) => Angle(0),
        (0, 1) => Angle(90),
        (-1, 0) => Angle(180),
        (0, -1) => Angle(270),
        _ => Angle(0),
    }
}

#[cfg(test)]
mod tests {
    use super::direction_of;
    use crate::geometry::{Angle, Transform};
    use crate::model::items::Mirror;

    #[test]
    fn a_pin_direction_follows_its_symbol() {
        // A pin drawn pointing up in the library points up on an unrotated
        // symbol, which is 270 in schematic sense because Y grows downwards.
        let identity = Transform::default();
        assert_eq!(direction_of(Angle(90), identity), Angle(270));
        assert_eq!(direction_of(Angle(0), identity), Angle(0));
        assert_eq!(direction_of(Angle(180), identity), Angle(180));
        assert_eq!(direction_of(Angle(270), identity), Angle(90));

        // Rotating the symbol by 90 degrees turns every pin with it.
        let rotated = Transform::from_file(Angle(90), None);
        assert_eq!(direction_of(Angle(0), rotated), Angle(270));

        // A mirror reverses the axis it acts on and leaves the other alone.
        let mirrored = Transform::from_file(Angle(0), Some(Mirror::Y));
        assert_eq!(direction_of(Angle(0), mirrored), Angle(180));
        assert_eq!(direction_of(Angle(90), mirrored), Angle(270));
    }
}
