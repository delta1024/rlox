use std::iter::Peekable;
pub(super) struct PeekableNext<I: Iterator> {
    iter: Peekable<I>,
    peek_value: Option<I::Item>,
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
            peek_value: None,
        }
    }
}
impl<I: Iterator> Iterator for PeekableNext<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
	match self.peek_value.take() {
	    None => self.iter.next(),
	    Some(a)	     => Some(a),
	}
    }
}

impl<I: Iterator> PeekableNext<I> {
    pub(super) fn peek(&mut self) -> Option<&<I as Iterator>::Item> {
        match self.peek_value.as_ref() {
            None => self.iter.peek(),
            Some(a) => Some(a),
        }
    }
    pub(super) fn peek_mut(&mut self) -> Option<&mut <I as Iterator>::Item> {
        match self.peek_value.as_mut() {
            None => self.iter.peek_mut(),
            Some(a) => Some(a),
        }
    }
    pub(super) fn peek_next(&mut self) -> Option<&<I as Iterator>::Item> {
        if self.peek_value.is_some() {
            self.iter.peek()
        } else {
            self.peek_value = self.iter.next();
            self.iter.peek()
        }
    }
    pub(super) fn next_if(
        &mut self,
        func: impl FnOnce(&<I as Iterator>::Item) -> bool,
    ) -> Option<<I as Iterator>::Item> {
        match self.peek_value.as_ref() {
            None => self.iter.next_if(func),
            Some(a) if func(a) => self.next(),
            Some(_) => None,
        }
    }
    pub(super) fn next_if_equal<T>(&mut self, expected: &T) -> Option<<I as Iterator>::Item>
    where
        <I as Iterator>::Item: PartialEq<T>,
        T: ?Sized,
    {
	match self.peek_value.as_ref() {
	    None => self.iter.next_if_eq(expected),
	    Some(a) if a == expected => self.next(),
	    Some(_) => None,
	}
    }
}
