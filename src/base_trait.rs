use std::iter::*;
use std::ops::*;

/* ---------------------------------------- Iterator ---------------------------------------- */
// Reuses efficient algorithm from `std::slice::Iter`.

#[derive(Debug, Clone)]
pub struct StringSequenceIter<'a> {
    buffer: &'a [u8],
    index: std::slice::Iter<'a, std::ops::Range<u32>>,
}

impl<'a> Iterator for StringSequenceIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index.next().cloned().map(|range| retr(self.buffer, range))
    }

    /* ------------------------------------- Forwarding ------------------------------------- */
    // NOTE: These are just forwarding methods from slice iterator

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.index.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.index.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index.nth(n).cloned().map(|range| retr(self.buffer, range))
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.index.fold(init, move |acc, range| f(acc, retr(self.buffer, range.clone())))
    }
}

impl<'a> DoubleEndedIterator for StringSequenceIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.index.next_back().cloned().map(|range| retr(self.buffer, range))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.index.nth_back(n).cloned().map(|range| retr(self.buffer, range))
    }
}

impl<'a> ExactSizeIterator for StringSequenceIter<'a> {}

impl<'a> FusedIterator for StringSequenceIter<'a> {}

/* ---------------------------------- Range32 To RangeUsize --------------------------------- */

pub(crate) fn up(x: Range<u32>) -> Range<usize> {
    x.start as usize..x.end as usize
}

pub(crate) fn retr(buf: &[u8], range: Range<u32>) -> &str {
    // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
    unsafe { std::str::from_utf8_unchecked(&buf[up(range)]) }
}

/* -------------------------------------- Primary Trait ------------------------------------- */

/// Viewer functionality for `StringSequence`.
pub(crate) trait StringSequenceView {
    fn inner(&self) -> (&[u8], &[Range<u32>]);

    fn iter(&self) -> StringSequenceIter {
        let (buffer, index) = self.inner();
        StringSequenceIter { buffer, index: index.iter() }
    }

    fn slice(&self, range: impl ToRange) -> StringSequenceIter {
        let (buffer, index) = self.inner();
        StringSequenceIter { buffer, index: index[range.to_range(index.len())].iter() }
    }

    fn text(&self) -> &str {
        let (buffer, _) = self.inner();
        // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
        unsafe { std::str::from_utf8_unchecked(buffer) }
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.text())
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/* ------------------------------------ Range Conversion ------------------------------------ */

/// Supports conversion to valid range.
///
/// > Basically does `NOT` verify that the specified range component is valid, as I know that is the
/// > default behavior of the `Index` trait. Please correct me if I'm wrong!
pub trait ToRange {
    fn to_range(self, array_len: usize) -> Range<usize>;
}

impl ToRange for Range<usize> {
    fn to_range(self, _: usize) -> Range<usize> {
        self
    }
}

impl ToRange for RangeFrom<usize> {
    fn to_range(self, array_len: usize) -> Range<usize> {
        self.start..array_len
    }
}

impl ToRange for RangeTo<usize> {
    fn to_range(self, _: usize) -> Range<usize> {
        0..self.end
    }
}

impl ToRange for RangeFull {
    fn to_range(self, array_len: usize) -> Range<usize> {
        0..array_len
    }
}
