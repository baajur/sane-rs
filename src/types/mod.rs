mod std;
pub use self::std::*;
use std::io::Read;

use error::Error;
use {Result, TryFromStream};

/// I made a different version of Option because the SANE devs are _special_.
/// Who else would make a protocol where, in some instances, a word with the
/// value of `1` indicates the subsequent value "is null", while in other
/// instances, a word with a value of `0` indicidates the pointer value is null?
pub enum Pointer<T> {
    Some(T),
    Null,
}

impl<T> TryFromStream for Pointer<T>
where
    T: TryFromStream,
{
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        let is_null = i32::try_from_stream(stream)?;

        match is_null {
            0 => Ok(Pointer::Null),
            _ => Ok(Pointer::Some(T::try_from_stream(stream)?)),
        }
    }
}

impl<T> Pointer<T> {
    /// Applies a function to the contained value (if any),
    /// or returns a [`default`][] (if not).
    ///
    /// [`default`]: ../default/trait.Default.html#tymethod.default
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Pointer::Some("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let x: Pointer<&str> = Pointer::Null;
    /// assert_eq!(x.map_or(42, |v| v.len()), 42);
    /// ```
    #[inline]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            Pointer::Some(t) => f(t),
            Pointer::Null => default,
        }
    }
}

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
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
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
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
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
    Range(Option<Range>),
}

#[derive(Debug)]
pub struct Range {
    min: i32,
    max: i32,
    quant: i32,
}

impl TryFromStream for Range {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        Ok(Range {
            min: i32::try_from_stream(stream)?,
            max: i32::try_from_stream(stream)?,
            quant: i32::try_from_stream(stream)?,
        })
    }
}

impl TryFromStream for Option<StringListConstraint> {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(None), // There is no constraint
            3 => {
                let opts = <Vec<Option<String>>>::try_from_stream(stream).map(|str_list| {
                    str_list.into_iter()
                        // Filter out any None strings
                        .filter(|s| s.is_some())
                        // None Strings are gone, so unwrap all values
                        .map(|s| s.unwrap()).collect()
                })?;
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
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        // See: http://www.sane-project.org/html/doc011.html#s4.2.9.8
        match i32::try_from_stream(stream)? {
            0 => Ok(None), // There is no constraint
            1 => Ok(Some(NumericalConstraint::Range(<_>::try_from_stream(
                stream,
            )?))),
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
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
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
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
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

impl OptionDescriptor {
    pub fn size(&self) -> i32 {
        match self {
            &OptionDescriptor::Boolean { .. } => 4,
            &OptionDescriptor::Integer { size, .. } | &OptionDescriptor::Fixed { size, .. } => size,
            &OptionDescriptor::String { max_length, .. } => max_length,
            _ => 0,
        }
    }

    pub fn read_value<S: Read>(&self, stream: &mut S) -> Result<ControlOptionResult> {
        let info = ControlOptionSetInfo::try_from_stream(stream)?;
        println!("info is {:?}, Checking value type", info);
        let value_type = i32::try_from_stream(stream)?;
        println!("type is {}, checking value size", value_type);
        let value_size = i32::try_from_stream(stream)?;
        println!("Value size is {}, reading value", value_size);

        assert_eq!(self.size(), value_size);

        // The value is a pointer, so read if it is null or not.
        let is_null = u32::try_from_stream(stream)? == 0;

        let value = match is_null {
            false => Some(match self {
                &OptionDescriptor::Boolean { .. } => {
                    OptionValue::Boolean(bool::try_from_stream(stream)?)
                }
                &OptionDescriptor::Integer { .. } => {
                    OptionValue::Integer(i32::try_from_stream(stream)?)
                }
                &OptionDescriptor::Fixed { .. } => {
                    OptionValue::Fixed(i32::try_from_stream(stream)?)
                }
                &OptionDescriptor::String { .. } => {
                    OptionValue::String(<_>::try_from_stream(stream)?)
                }
                &OptionDescriptor::Button { .. } => OptionValue::Button,
                &OptionDescriptor::Group { .. } => OptionValue::Group,
            }),
            true => None,
        };

        Ok(ControlOptionResult { value, info })
    }
}

impl TryFromStream for OptionDescriptor {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        let name: Option<String> = <_>::try_from_stream(stream)?;
        let title: Option<String> = <_>::try_from_stream(stream)?;
        let description: Option<String> = <_>::try_from_stream(stream)?;

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

/// Get the serialized type value of the type of the descriptor
impl<'a> From<&'a OptionDescriptor> for i32 {
    fn from(o: &'a OptionDescriptor) -> i32 {
        match o {
            &OptionDescriptor::Boolean { .. } => 0,
            &OptionDescriptor::Integer { .. } => 1,
            &OptionDescriptor::Fixed { .. } => 2,
            &OptionDescriptor::String { .. } => 3,
            &OptionDescriptor::Button { .. } => 4,
            &OptionDescriptor::Group { .. } => 5,
        }
    }
}

#[derive(Debug)]
pub enum OptionValue {
    Boolean(bool),
    Integer(i32),
    Fixed(i32),
    String(Option<String>),
    Button,
    Group,
}

bitflags!{
    #[derive(Default)]
    pub struct ControlOptionSetInfo: u32 {
        /// The setting of an option value resulted in a value being selected
        /// that does not exactly match the requested value.
        ///
        /// # Example
        ///
        /// > For example, if a scanner can adjust the resolution in increments
        /// > of 30dpi only, setting the resolution to 307dpi may result in an
        /// > actual setting of 300dpi.
        const Inexact       = 0x00000001;

        /// > The setting of an option may effect the value or availability of one
        /// > or more _other_ options. This indicates the application should
        /// > reload all options.
        ///
        /// > This may be set if and only if at least one option changed.
        const ReloadOptions = 0x00000002;

        /// > The setting of an option may affect the parameter values.
        ///
        /// > Note: this may be set even if the parameters did not actually change.
        /// > However, it is guaranteed that the parameters never change without
        /// > this value being set.
        const ReloadParams  = 0x00000004;
    }
}

impl TryFromStream for ControlOptionSetInfo {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        Ok(ControlOptionSetInfo::from_bits_truncate(
            <u32>::try_from_stream(stream)?,
        ))
    }
}

#[derive(Debug)]
pub struct ControlOptionResult {
    value: Option<OptionValue>,
    info: ControlOptionSetInfo,
}
