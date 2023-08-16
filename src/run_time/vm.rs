use crate::{
    byte_code::OpCode,
    heap::{Allocator, ObjPtr, ObjString, Object},
    run_time::{RuntimeError, RuntimeState},
    runtime_error,
    stack::Stack,
    value::Value,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BinaryOp {
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    Equal(Value, Value),
    Greater(Value, Value),
    Less(Value, Value),
}

impl BinaryOp {
    pub(crate) fn new(op_code: OpCode, a: Value, b: Value) -> Self {
        match op_code {
            OpCode::Add => Self::Add(a, b),
            OpCode::Sub => Self::Sub(a, b),
            OpCode::Mul => Self::Mul(a, b),
            OpCode::Div => Self::Div(a, b),
            OpCode::Equal => Self::Equal(a, b),
            OpCode::Greater => Self::Greater(a, b),
            OpCode::Less => Self::Less(a, b),
            _ => unreachable!(),
        }
    }
}

pub(crate) enum UnaryOp {
    Negate(Value),
    Not(Value),
}
impl UnaryOp {
    pub(crate) fn new(op: OpCode, a: Value) -> Self {
        match op {
            OpCode::Neg => Self::Negate(a),
            OpCode::Not => Self::Not(a),
            _ => unreachable!(),
        }
    }
}

pub type VmResult<T> = std::result::Result<T, RuntimeError>;

pub(crate) struct Vm {
    pub(crate) stack: Stack<Value>,
    pub(crate) allocator: Allocator,
}

impl Vm {
    pub(crate) fn new(allocator: Allocator) -> Self {
        Self {
            stack: Stack::new(),
            allocator,
        }
    }
    #[inline(always)]
    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    #[inline(always)]
    pub(crate) fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
    pub(crate) fn binary_instruction<'a, 'b>(
        state: &mut RuntimeState<'a, 'b>,
        instruction: BinaryOp,
    ) -> VmResult<Value> {
        let num = match instruction {
            BinaryOp::Add(Value::Number(a), Value::Number(b)) => a + b,
            BinaryOp::Add(Value::Object(a), Value::Object(b))
                if a.is_obj::<ObjString>() && b.is_obj::<ObjString>() =>
            {
                return Ok(Vm::concatenate(state, a, b))
            }
            BinaryOp::Add(_, _) => {
                runtime_error!(state, "Operands must be two numbers or two strings")
            }
            BinaryOp::Sub(Value::Number(a), Value::Number(b)) => a - b,
            BinaryOp::Mul(Value::Number(a), Value::Number(b)) => a * b,
            BinaryOp::Div(Value::Number(a), Value::Number(b)) => a / b,

            BinaryOp::Less(Value::Number(a), Value::Number(b)) => return Ok(Value::Bool(a < b)),
            BinaryOp::Greater(Value::Number(a), Value::Number(b)) => return Ok(Value::Bool(a > b)),
            BinaryOp::Equal(a, b) => return Ok(Value::Bool(a == b)),
            _ => runtime_error!(state, "Operands must be two numbers."),
        };
        Ok(Value::Number(num))
    }
    pub(crate) fn unary_instruction<'a, 'b>(
        state: &mut RuntimeState<'a, 'b>,
        instruction: UnaryOp,
    ) -> VmResult<Value> {
        Ok(match instruction {
            UnaryOp::Negate(Value::Number(a)) => Value::Number(-a),
            UnaryOp::Negate(_) => runtime_error!(state, "Operand must be a number"),
            UnaryOp::Not(v) => !v,
        })
    }
    pub(crate) fn concatenate<'a, 'b>(
        state: &mut RuntimeState<'a, 'b>,
        a: Object,
        b: Object,
    ) -> Value {
        let (a, b) = (a.as_obj::<ObjString>(), b.as_obj::<ObjString>());
        let (a, b) = {
            let a = a.as_ref();
            let b = b.as_ref();
            let a_len = a[..].len() - 1;
            // remove the trailing and leading '"' from a and b respectivly
            (&a[..a_len], &b[1..])
        };
        let result = format!("{a}{b}");
        let obj = state.vm.allocator.allocate_string(result);
        obj.into()
    }
}
