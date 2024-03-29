use crate::{
    heap::{ObjPtr, ObjString},
    value::Value,
};

#[derive(Debug, Copy, Clone)]
pub(crate) enum OpCode {
    Return,
    Constant(Value),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal(ObjPtr<ObjString>),
    GetGlobal(ObjPtr<ObjString>),
    SetGlobal(ObjPtr<ObjString>),
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Return,
            1 => Self::Constant(Value::default()),
            2 => Self::Add,
            3 => Self::Sub,
            4 => Self::Mul,
            5 => Self::Div,
            6 => Self::Neg,
            7 => Self::Nil,
            8 => Self::True,
            9 => Self::False,
            10 => Self::Not,
            11 => OpCode::Equal,
            12 => OpCode::Greater,
            13 => OpCode::Less,
            14 => OpCode::Print,
            15 => OpCode::Pop,
            _ => unreachable!(),
        }
    }
}
impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Return => 0,
            OpCode::Constant(_) => 1,
            OpCode::Add => 2,
            OpCode::Sub => 3,
            OpCode::Mul => 4,
            OpCode::Div => 5,
            OpCode::Neg => 6,
            OpCode::Nil => 7,
            OpCode::True => 8,
            OpCode::False => 9,
            OpCode::Not => 10,
            OpCode::Equal => 11,
            OpCode::Greater => 12,
            OpCode::Less => 13,
            OpCode::Print => 14,
            OpCode::Pop => 15,
            OpCode::DefineGlobal(_) => 16,
            OpCode::GetGlobal(_) => 17,
            OpCode::SetGlobal(_) => 18,
        }
    }
}
pub(super) const OP_CODE_MAX: u8 = 15;
