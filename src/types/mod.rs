mod std;
pub use self::std::*;

use error::Error;
use {Result, TryFromStream};
use std::net::TcpStream;

/// The type of an option value, in an OptionDescriptor.
///
/// See: http://www.sane-project.org/html/doc011.html#s4.2.9.4
pub enum OptionValueType {
    Boolean,
    Integer,
    Fixed,
    String,
    /// An option of this type has no value. Instead, setting an option of this
    /// type has an option-specific side-effect. For example, a button-typed
    /// option could be used by a backend to provide a means to select default
    /// values or to the tell an automatic document feeder to advance to the
    /// next sheet of paper.
    Button,

    /// An option of this type has no value. This type is used to group
    /// logically related options. A group option is in effect up to the point
    /// where another group option is encountered (or up to the end of
    /// the option list, if there are no other group options).
    ///
    /// For group options, only members `title` and `type` are valid
    /// in the option descriptor.
    Group,
}

impl TryFromStream for OptionValueType {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.4
        match i32::try_from_stream(stream)? {
            0 => Ok(OptionValueType::Boolean),
            1 => Ok(OptionValueType::Integer),
            2 => Ok(OptionValueType::Fixed),
            3 => Ok(OptionValueType::String),
            4 => Ok(OptionValueType::Button),
            5 => Ok(OptionValueType::Group),
            x => Err(Error::InvalidSaneFieldValue(
                "Received invalid value for OptionValueType field".into(),
                x,
            )),
        }
    }
}

/// The physical unit of an option.
///
/// > Note that the specified unit is what the SANE backend expects.
/// > It is entirely up to a frontend as to how these units a presented to the user.
/// > For example, SANE expresses all lengths in millimeters. A frontend is generally
/// > expected to provide appropriate conversion routines so that a user can express
/// > quantities in a customary unit (e.g., inches or centimeters).
///
/// See: http://www.sane-project.org/html/doc011.html#s4.2.9.5
pub enum OptionUnit {
    None,
    Pixel,
    Bit,
    Millimeter,
    DPI,
    Percent,
    Microsecond,
}

impl TryFromStream for OptionUnit {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.5
        match i32::try_from_stream(stream)? {
            0 => Ok(OptionUnit::None),
            1 => Ok(OptionUnit::Pixel),
            2 => Ok(OptionUnit::Bit),
            3 => Ok(OptionUnit::Millimeter),
            4 => Ok(OptionUnit::DPI),
            5 => Ok(OptionUnit::Percent),
            6 => Ok(OptionUnit::Microsecond),
            x => Err(Error::InvalidSaneFieldValue(
                "Received invalid value for OptionUnit field".into(),
                x,
            )),
        }
    }
}

/// Constrain the values at an option can take. For example, constraints can
/// be used by a frontend to determine how to represent a given option.
///
/// See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
pub enum Constraint {
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    Range { min: i32, max: i32, quant: i32 },
}

impl TryFromStream for Option<Constraint> {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(None), // There is no constraint
            1 => Ok(Some(Constraint::Range {
                min: i32::try_from_stream(stream)?,
                max: i32::try_from_stream(stream)?,
                quant: i32::try_from_stream(stream)?,
            })),
            2 => Ok(Some(Constraint::IntegerList(<_>::try_from_stream(stream)?))),
            3 => Ok(Some(Constraint::StringList(<_>::try_from_stream(stream)?))),
            x => Err(Error::InvalidSaneFieldValue(
                "Received invalid value for Contraint field".into(),
                x,
            )),
        }
    }
}

/*
pub struct OptionDescriptor {
    name: String,
    title: String,
    desciption: String,
    kind: OptionValueType,
    unit: Unit,
    size: i32,
    cap: i32,
    constraint: Constraint,
}
*/
