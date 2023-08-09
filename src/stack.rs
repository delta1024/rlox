use std::ops::{Add, AddAssign, Sub, SubAssign};

pub(crate) const STACK_MAX: usize = 256;
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct StackTop(usize);

impl From<usize> for StackTop {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Add<usize> for StackTop {
    type Output = usize;
    fn add(self, rhs: usize) -> Self::Output {
        self.0 + rhs
    }
}
impl Sub<usize> for StackTop {
    type Output = usize;
    fn sub(self, rhs: usize) -> Self::Output {
        self.0 - rhs
    }
}
impl AddAssign for StackTop {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl SubAssign for StackTop {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
pub(crate) struct Stack<T: Copy> {
    stack_top: StackTop,
    data: [Option<T>; STACK_MAX],
}

impl<T: Copy> Stack<T> {
    pub(crate) fn new() -> Self {
        Self {
            stack_top: StackTop::default(),
            data: [None; STACK_MAX],
        }
    }
    /// # Panics
    /// will panic if a len of [`STACK_TOP`] is exceded.
    pub(crate) fn push(&mut self, value: T) {
        if self.stack_top.0 == STACK_MAX {
            panic!("stack overflow");
        }
        self.stack_top += 1.into();
        self.data[self.stack_top - 1] = Some(value);
    }
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.stack_top.0 > 0 {
            self.stack_top -= 1.into();
        }
        self.data[self.stack_top.0].take()
    }
    pub(crate) fn reset(&mut self) {
        self.stack_top = 0.into();
    }
}
