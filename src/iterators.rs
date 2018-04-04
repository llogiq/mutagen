
/// NoopIterator is an iterator that will return `None`
pub struct NoopIterator<I: Iterator> {
    pub inner: I,
}

impl<I: Iterator> Iterator for NoopIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        None
    }
}

pub struct SkipLast<I: Iterator> {
    /// Contains the inner iterator
    pub inner: I,
    /// Stashed contains the following element to be returned on the iterator. We need this value,
    /// as we need to advance the inner iterator one step ahead, to be able to check when the
    /// inner iterator ends. If this is the case, we need to ignore the stashed value.
    stashed: Option<I::Item>,
}

impl<I: Iterator> SkipLast<I> {
    pub fn new(mut inner: I) -> Self {
        let stashed = inner.next();

        SkipLast {
            inner,
            stashed,
        }
    }
}

impl<I: Iterator> Iterator for SkipLast<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let current = self.stashed.take();
        let next = self.inner.next();

        if next.is_none() {
            None
        } else {
            self.stashed = next;

            current
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_last_iterator() {
        let examples = vec![
            (vec![1, 2, 3, 4, 5], vec![1, 2, 3, 4]),
            (vec![1, 2, 3, 4], vec![1, 2, 3]),
            (vec![1, 2, 3], vec![1, 2]),
            (vec![1, 2], vec![1]),
            (vec![1], Vec::<i32>::new()),
            (vec![], Vec::<i32>::new()),
        ];

        for e in examples {
            let skip_bound = SkipLast::new(e.0.iter());
            let skipped: Vec<i32> = skip_bound.cloned().collect();

            assert_eq!(e.1, skipped);
        }
    }
}
