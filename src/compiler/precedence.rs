use std::ops::Add;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub(super) enum Precedence {
    #[default]
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Assignment,
            2 => Self::Or,
            3 => Self::And,
            4 => Self::Equality,
            5 => Self::Comparison,
            6 => Self::Term,
            7 => Self::Factor,
            8 => Self::Unary,
            9 => Self::Call,
            10 => Self::Primary,

            _ => unreachable!(),
        }
    }
}

impl Add<usize> for Precedence {
    type Output = Precedence;
    fn add(self, rhs: usize) -> Self::Output {
        Precedence::from((self as usize) + rhs)
    }
}
