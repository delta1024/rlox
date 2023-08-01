use std::iter::Peekable;
pub struct PeekableNext<I: Iterator> {
    iter: Peekable<I>,
    peek_value: Option<I::Item>,
}

pub trait PeekNext<I: Iterator> {
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
            Some(a) => Some(a),
        }
    }
}

impl<I: Iterator> PeekableNext<I> {
    /// Returns a reference to the next() value without advancing the iterator.
    ///
    /// Like [next](std::iter::Iterator::next), if there is a value, it is wrapped in a Some(T).
    /// But if the iteration is over, None is returned.
    ///
    /// Because peek() returns a reference, and many iterators iterate over
    /// references, there can be a possibly confusing situation where the
    /// return value is a double reference. You can see this effect in the
    /// examples below.
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use peekable_next::PeekNext;
    /// let xs = [1, 2, 3];
    ///
    /// let mut iter = xs.iter().peekable_next();
    ///
    /// // peek() lets us see into the future
    /// assert_eq!(iter.peek(), Some(&&1));
    /// assert_eq!(iter.next(), Some(&1));
    ///
    /// assert_eq!(iter.next(), Some(&2));
    ///
    /// // The iterator does not advance even if we `peek` multiple times
    /// assert_eq!(iter.peek(), Some(&&3));
    /// assert_eq!(iter.peek(), Some(&&3));
    ///
    /// assert_eq!(iter.next(), Some(&3));
    ///
    /// // After the iterator is finished, so is `peek()`
    /// assert_eq!(iter.peek(), None);
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn peek(&mut self) -> Option<&<I as Iterator>::Item> {
        match self.peek_value.as_ref() {
            None => self.iter.peek(),
            Some(a) => Some(a),
        }
    }
    /// Returns a mutable reference to the next() value without advancing the iterator.
    ///
    ///	Like [next](std::iter::Iterator::next), if there is a value, it is wrapped in a Some(T). But if the iteration is over, None is returned.
    ///
    ///	Because peek_mut() returns a reference, and many iterators
    ///	iterate over references, there can be a possibly confusing
    ///	situation where the return value is a double reference. You
    /// an see this effect in the examples below.
    /// # Examples
    ///
    ///	Basic usage:
    /// ```
    /// use peekable_next::PeekNext;
    ///
    /// let mut iter = [1, 2, 3].iter().peekable();
    ///
    /// // Like with `peek()`, we can see into the future without advancing the iterator.
    /// assert_eq!(iter.peek_mut(), Some(&mut &1));
    /// assert_eq!(iter.peek_mut(), Some(&mut &1));
    /// assert_eq!(iter.next(), Some(&1));
    ///
    /// // Peek into the iterator and set the value behind the mutable reference.
    /// if let Some(p) = iter.peek_mut() {
    ///	  assert_eq!(*p, &2);
    ///   *p = &5;
    /// }
    ///
    /// // The value we put in reappears as the iterator continues.
    /// assert_eq!(iter.collect::<Vec<_>>(), vec![&5, &3]);
    /// ```

    pub fn peek_mut(&mut self) -> Option<&mut <I as Iterator>::Item> {
        match self.peek_value.as_mut() {
            None => self.iter.peek_mut(),
            Some(a) => Some(a),
        }
    }
    /// Returns a referencs to the value after next() without
    /// advancing the iterator.
    ///
    /// Like [next](std::iter::Iterator::next), if there is a value, it is wrapped in a
    /// Some(T). But if the iteration is over, None is returned.
    ///
    /// Because peek_next() returns a reference, and many iterators
    /// iterate over references, there can be a possibly confusing
    /// situation where the return value is a double reference. You
    /// can see this effect in the examples below.
    ///
    /// # Examples
    ///
    /// Basics usage:
    ///
    /// ```
    /// use peekable_next::PeekNext;
    /// let xs = [1, 2, 3];
    ///
    /// let mut iter = xs.iter().peekable_next();
    ///
    /// // peek() lets us see into the future
    /// assert_eq!(iter.peek(), Some(&&1));
    /// assert_eq!(iter.next(), Some(&1));
    ///
    /// // peek_next() lets us see even further into the future
    /// assert_eq!(iter.peek_next(), Some(&&3));
    ///
    /// assert_eq!(iter.next(), Some(&2));
    ///
    /// // The iterator does not advance even if we `peek` multiple times
    /// assert_eq!(iter.peek(), Some(&&3));
    /// assert_eq!(iter.peek(), Some(&&3));
    ///
    /// assert_eq!(iter.next(), Some(&3));
    ///
    /// // After the iterator is finished, so is `peek()`
    /// assert_eq!(iter.peek(), None);
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn peek_next(&mut self) -> Option<&<I as Iterator>::Item> {
        if self.peek_value.is_some() {
            self.iter.peek()
        } else {
            self.peek_value = self.iter.next();
            self.iter.peek()
        }
    }
    /// Consume and return the next value of this iterator if a condition is true.
    ///
    /// If func returns true for the next value of this iterator, consume and return it. Otherwise, return None.
    /// # Examples
    ///
    /// Consume a number if it’s equal to 0.
    /// ```
    ///use peekable_next::PeekNext;
    ///
    /// let mut iter = (0..5).peekable_next();
    /// // The first item of the iterator is 0; consume it.
    /// assert_eq!(iter.next_if(|&x| x == 0), Some(0));
    /// // The next item returned is now 1, so `consume` will return `false`.
    /// assert_eq!(iter.next_if(|&x| x == 0), None);
    /// // `next_if` saves the value of the next item if it was not equal to `expected`.
    /// assert_eq!(iter.next(), Some(1));
    /// ```
    pub fn next_if(
        &mut self,
        func: impl FnOnce(&<I as Iterator>::Item) -> bool,
    ) -> Option<<I as Iterator>::Item> {
        match self.peek_value.as_ref() {
            None => self.iter.next_if(func),
            Some(a) if func(a) => self.next(),
            Some(_) => None,
        }
    }
    /// Consume and return the next item if it is equal to expected.
    ///
    ///	# Example
    ///
    ///	Consume a number if it’s equal to 0.
    ///
    /// ```
    /// use peekable_next::PeekNext;
    ///	let mut iter = (0..5).peekable_next();
    ///    // The first item of the iterator is 0; consume it.
    ///    assert_eq!(iter.next_if_eq(&0), Some(0));
    ///    // The next item returned is now 1, so `consume` will return `false`.
    ///    assert_eq!(iter.next_if_eq(&0), None);
    ///    // `next_if_eq` saves the value of the next item if it was not equal to `expected`.
    ///    assert_eq!(iter.next(), Some(1));
    /// ```
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<<I as Iterator>::Item>
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
