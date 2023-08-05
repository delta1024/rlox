pub(crate) mod pc {
    use std::ops::{Add, AddAssign, Deref, DerefMut};

    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub(crate) struct PositionCounter(usize);

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
    impl AddAssign for PositionCounter {
        fn add_assign(&mut self, rhs: Self) {
            let n = *self + *rhs;
            *self = n;
        }
    }
}
