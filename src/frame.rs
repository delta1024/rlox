use crate::byte_code::{Chunk, OpCode};

use self::pc::PositionCounter;

pub(crate) mod pc {
    use std::ops::{Add, AddAssign, Deref, DerefMut, Sub};

    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
    pub(crate) struct PositionCounter(usize);
    impl From<usize> for PositionCounter {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }
    impl Deref for PositionCounter {
        type Target = usize;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl DerefMut for PositionCounter {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl Add<usize> for PositionCounter {
        type Output = PositionCounter;
        fn add(self, rhs: usize) -> Self::Output {
            PositionCounter(*self + rhs)
        }
    }
    impl Sub<usize> for PositionCounter {
        type Output = PositionCounter;
        fn sub(self, rhs: usize) -> Self::Output {
            PositionCounter(*self - rhs)
        }
    }
    impl AddAssign for PositionCounter {
        fn add_assign(&mut self, rhs: Self) {
            let n = *self + *rhs;
            *self = n;
        }
    }
}

pub(crate) struct CallFrame<'a> {
    chunk: &'a Chunk,
    pub(crate) position_conunter: PositionCounter,
}

impl<'a> CallFrame<'a> {
    pub(crate) fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            position_conunter: 0.into(),
        }
    }
    pub(crate) fn advance_position(&mut self) -> OpCode {
        let (op, pos) = self.chunk.get_instruction(self.position_conunter);
        self.position_conunter += pos;
        op
    }
}
