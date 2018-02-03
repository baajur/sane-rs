mod std;
pub use self::std::*;

use error::Error;
use {Result, TryFromStream};
use std::net::TcpStream;

/// The type of an option value, in an OptionDescriptor.
///
/// See: http://www.sane-project.org/html/doc011.html#s4.2.9.4
#[derive(Debug)]
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
#[derive(Debug)]
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

pub trait OptionConstraint {}

#[derive(Debug)]
pub struct NoConstraint;
#[derive(Debug)]
pub struct StringListConstraint(Vec<String>);
#[derive(Debug)]
pub enum NumericalConstraint {
    IntegerList(Vec<i32>),
    Range { min: i32, max: i32, quant: i32 },
}

impl TryFromStream for Option<StringListConstraint> {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(None), // There is no constraint
            3 => {
                let opts = ::read_string_array(stream)?;
                debug!("String constraint options: {:?}", opts);
                Ok(Some(StringListConstraint(opts)))
            }
            x => Err(Error::InvalidSaneFieldValue(
                "Received invalid value for String Contraint field".into(),
                x,
            )),
        }
    }
}

impl TryFromStream for Option<NumericalConstraint> {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(None), // There is no constraint
            1 => Ok(Some(NumericalConstraint::Range {
                min: i32::try_from_stream(stream)?,
                max: i32::try_from_stream(stream)?,
                quant: i32::try_from_stream(stream)?,
            })),
            2 => Ok(Some(NumericalConstraint::IntegerList(
                <_>::try_from_stream(stream)?,
            ))),
            x => Err(Error::InvalidSaneFieldValue(
                "Received invalid value for Numerical Contraint field".into(),
                x,
            )),
        }
    }
}

impl TryFromStream for NoConstraint {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(NoConstraint), // There is no constraint
            x => Err(Error::InvalidSaneFieldValue(
                "Received a constraint on an option field that should not have constraints!".into(),
                x,
            )),
        }
    }
}

bitflags!{
    #[derive(Default)]
    pub struct Capabilities: u32 {
        /// The option value can be set by a call to `sane_control_option()`.
        const SoftSelect = 0b00000001;

        /// The option value can be set by user-intervention (e.g., by flipping a switch).
        /// The user-interface should prompt the user to execute the appropriate action
        /// to set such an option. This capability is mutually exclusive with `SoftSelect`
        /// (either one of them can be set, but not both simultaneously).
        const HardSelect = 0b00000010;

        /// The option value can be detected by software. If `SoftSelect` is set,
        /// this capability must be set. If `HardSelect` is set, this capability may or
        /// may not be set. If this capability is set but neither `SoftSelect` nor
        /// `HardSelect` are, then there is no way to control the option.
        /// That is, the option provides read-out of the current value only.
        const SoftDetect = 0b00000100;

        /// If set, this capability indicates that an option is not directly
        /// supported by the device and is instead emulated in the backend.
        /// A sophisticated frontend may elect to use its own (presumably better)
        /// emulation in lieu of an emulated option.
        const Emulated   = 0b00001000;

        /// If set, this capability indicates that the backend (or the device)
        /// is capable to picking a reasonable option value automatically.
        /// For such options, it is possible to select automatic operation
        /// by calling `sane_control_option()` with an action value of SANE_ACTION_SET_AUTO.
        const Automatic  = 0b00010000;

        /// If set, this capability indicates that the option is not currently active
        /// (e.g., because it's meaningful only if another option is set to some other value).
        const Inactive   = 0b00100000;

        /// If set, this capability indicates that the option should be considered
        /// an "advanced user option."" A frontend typically displays such options
        /// in a less conspicuous way than regular options;
        /// (e.g., a command line interface may list such options last or
        /// a graphical interface may make them available in a seperate
        /// "advanced settings" dialog).
        const Advanced   = 0b01000000;
    }
}

impl TryFromStream for Capabilities {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        Ok(Capabilities::from_bits_truncate(<u32>::try_from_stream(
            stream,
        )?))
    }
}

#[derive(Debug)]
pub enum OptionDescriptor {
    Boolean {
        name: String,
        title: String,
        description: String,
        unit: OptionUnit,
        // size is 4 bytes
        capabilities: Capabilities,
        _no_constrainst: NoConstraint,
    },
    Integer {
        name: String,
        title: String,
        description: String,
        unit: OptionUnit,
        size: i32,
        capabilities: Capabilities,
        constraint: Option<NumericalConstraint>,
    },
    Fixed {
        name: String,
        title: String,
        description: String,
        unit: OptionUnit,
        size: i32,
        capabilities: Capabilities,
        constraint: Option<NumericalConstraint>,
    },
    String {
        name: String,
        title: String,
        description: String,
        unit: OptionUnit,
        max_length: i32,
        capabilities: Capabilities,
        constraint: Option<StringListConstraint>,
    },
    Button {
        name: String,
        title: String,
        description: String,
        unit: OptionUnit,
        // size is ignored
        capabilities: Capabilities,
        _no_constrainst: NoConstraint,
    },
    Group {
        title: String,
        _no_constrainst: NoConstraint,
    },
}

impl TryFromStream for OptionDescriptor {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        let name: Option<String> = <_>::try_from_stream(stream)?;
        let title: Option<String> = <_>::try_from_stream(stream)?;
        let description: Option<String> = <_>::try_from_stream(stream)?;

        debug!("Name: {}", name.as_ref().unwrap_or(&"".into()));
        debug!("Title: {}", title.as_ref().unwrap_or(&"".into()));
        debug!(
            "Description: {}",
            description.as_ref().unwrap_or(&"".into())
        );

        let kind = OptionValueType::try_from_stream(stream)?;
        let unit = OptionUnit::try_from_stream(stream)?;
        let size = <i32>::try_from_stream(stream)?;
        let capabilities = Capabilities::try_from_stream(stream)?;

        // we'll read constraints later

        let opt = match kind {
            OptionValueType::Boolean => Ok(OptionDescriptor::Boolean {
                name: name?,
                title: title?,
                description: description?,
                unit,
                capabilities,
                _no_constrainst: NoConstraint::try_from_stream(stream)?,
            }),
            OptionValueType::Integer => Ok(OptionDescriptor::Integer {
                name: name?,
                title: title?,
                description: description?,
                unit,
                size,
                capabilities,
                constraint: <_>::try_from_stream(stream)?,
            }),
            OptionValueType::Fixed => Ok(OptionDescriptor::Fixed {
                name: name?,
                title: title?,
                description: description?,
                unit,
                size,
                capabilities,
                constraint: <_>::try_from_stream(stream)?,
            }),
            OptionValueType::String => Ok(OptionDescriptor::String {
                name: name?,
                title: title?,
                description: description?,
                unit,
                max_length: size,
                capabilities,
                constraint: <_>::try_from_stream(stream)?,
            }),
            OptionValueType::Button => Ok(OptionDescriptor::Button {
                name: name?,
                title: title?,
                description: description?,
                unit,
                capabilities,
                _no_constrainst: NoConstraint::try_from_stream(stream)?,
            }),
            OptionValueType::Group => Ok(OptionDescriptor::Group {
                title: title?,
                _no_constrainst: NoConstraint::try_from_stream(stream)?,
            }),
        };

        debug!("{:?}", opt);

        opt
    }
}
