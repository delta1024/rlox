use std::iter::Peekable;
pub(super) struct PeekableNext<I: Iterator> {
    iter: Peekable<I>,
    next: Option<I::Item>,
}

pub(super) trait PeekNext<I: Iterator> {
    fn peekable_next(self) -> PeekableNext<I>;
}

impl<I> PeekNext<I> for I
where
    I: Iterator,
{
    fn peekable_next(self) -> PeekableNext<I> {
        PeekableNext {
            iter: self.peekable(),
            next: None,
        }
    }
}
impl<I: Iterator> Iterator for PeekableNext<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
	todo!()
    }
}

impl<I: Iterator> PeekableNext<I> {
    pub(super) fn peek(&mut self) -> Option<&<I as Iterator>::Item> {
        todo!()
    }
    pub(super) fn peek_next(&mut self) -> Option<&<I as Iterator>::Item> {
        todo!()
    }
    pub(super) fn next_if(
        &mut self,
        func: impl FnOnce(&<I as Iterator>::Item) -> bool,
    ) -> Option<<I as Iterator>::Item> {
        todo!()
    }
    pub(super) fn next_if_equal<T>(&mut self, expected: &T) -> Option<<I as Iterator>::Item>
    where
        <I as Iterator>::Item: PartialEq<T>,
        T: ?Sized,
    {
        todo!()
    }
}
