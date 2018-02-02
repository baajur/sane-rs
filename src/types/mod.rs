mod std;
pub use self::std::*;

pub enum ValueType {
    Boolean,
    Integer,
    Fixed,
    String,
    Button,
    Group,
}

pub enum Unit {
    None,
    Pixel,
    Bit,
    MM,
    DPI,
    Percent,
    Microsecond,
}

pub enum Constraint {
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    Range { min: i32, max: i32, quant: i32 },
}

pub struct OptionDescriptor {
    name: String,
    title: String,
    desciption: String,
    kind: ValueType,
    unit: Unit,
    size: i32,
    cap: i32,
    constraint: Constraint,
}
