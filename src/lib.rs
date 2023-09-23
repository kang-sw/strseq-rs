macro_rules! impl_seq_view {
    ($Type:ident) => {
        /* ------------------------------------ Display Trait ----------------------------------- */
        impl std::fmt::Debug for $Type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as crate::base_trait::StringSequenceView>::fmt_debug(self, f)
            }
        }

        impl std::fmt::Display for $Type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as crate::base_trait::StringSequenceView>::fmt_display(self, f)
            }
        }

        /* ----------------------------------- Accessor Trait ----------------------------------- */
        impl std::ops::Index<usize> for $Type {
            type Output = str;

            fn index(&self, index: usize) -> &Self::Output {
                self.iter().nth(index).unwrap()
            }
        }

        /* ----------------------------------- Iterator Trait ----------------------------------- */
        impl std::hash::Hash for $Type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                std::hash::Hash::hash(self.text(), state)
            }
        }

        /* ---------------------------------------- Refs ---------------------------------------- */
        impl AsRef<str> for $Type {
            fn as_ref(&self) -> &str {
                self.text()
            }
        }

        impl AsRef<[u8]> for $Type {
            fn as_ref(&self) -> &[u8] {
                self.text().as_bytes()
            }
        }

        impl AsRef<std::path::Path> for $Type {
            fn as_ref(&self) -> &std::path::Path {
                std::path::Path::new(self.text())
            }
        }

        impl AsRef<std::ffi::OsStr> for $Type {
            fn as_ref(&self) -> &std::ffi::OsStr {
                std::ffi::OsStr::new(self.text())
            }
        }

        /* -------------------------------------- Type Impl ------------------------------------- */
        impl $Type {
            pub fn iter(&self) -> crate::base_trait::StringSequenceIter {
                <Self as crate::base_trait::StringSequenceView>::iter(self)
            }

            pub fn slice(
                &self,
                range: impl crate::base_trait::ToRange,
            ) -> crate::base_trait::StringSequenceIter {
                <Self as crate::base_trait::StringSequenceView>::slice(self, range)
            }

            pub fn get(&self, index: usize) -> Option<&str> {
                self.iter().nth(index)
            }

            pub fn text(&self) -> &str {
                <Self as crate::base_trait::StringSequenceView>::text(self)
            }

            pub fn tokens(&self) -> &[std::ops::Range<u32>] {
                let (_, index) = self.inner();
                index
            }

            pub fn front(&self) -> Option<&str> {
                self.get(0)
            }

            pub fn back(&self) -> Option<&str> {
                self.get(self.tokens().len() - 1)
            }

            pub fn len(&self) -> usize {
                self.tokens().len()
            }

            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }

            pub fn starts_with(&self, other: &[impl AsRef<str>]) -> bool {
                self.iter().zip(other.iter()).all(|(a, b)| a == b.as_ref())
            }

            pub fn ends_with(&self, other: &[impl AsRef<str>]) -> bool {
                self.iter().rev().zip(other.iter().rev()).all(|(a, b)| a == b.as_ref())
            }

            pub fn contains(&self, other: &[impl AsRef<str>]) -> bool {
                let mut iter = self.iter();

                if other.is_empty() {
                    return true;
                }

                loop {
                    if iter.len() < other.len() {
                        break false;
                    }

                    if iter.clone().take(other.len()).eq(other.iter().map(|x| x.as_ref())) {
                        break true;
                    }

                    iter.next();
                }
            }
        }

        impl<'a> IntoIterator for &'a $Type {
            type Item = &'a str;
            type IntoIter = crate::base_trait::StringSequenceIter<'a>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    };
}

#[doc(hidden)]
mod base_trait {
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

    fn retr<'a>(buf: &'a [u8], range: Range<u32>) -> &'a str {
        // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
        unsafe { std::str::from_utf8_unchecked(&buf[up(range)]) }
    }

    /* -------------------------------------- Primary Trait ------------------------------------- */

    /// Viewer functionality for `StringSequence`.
    pub(crate) trait StringSequenceView {
        fn inner(&self) -> (&[u8], &[Range<u32>]);

        fn iter(&self) -> StringSequenceIter {
            let (buffer, index) = self.inner();
            StringSequenceIter { buffer: buffer, index: index.iter() }
        }

        fn slice(&self, range: impl ToRange) -> StringSequenceIter {
            let (buffer, index) = self.inner();
            StringSequenceIter { buffer: buffer, index: index[range.to_range(index.len())].iter() }
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
            let (buffer, index) = self.inner();
            f.debug_struct(std::any::type_name::<Self>())
                .field("buffer", &buffer)
                .field("index", &index)
                .finish()
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
}
pub mod view {
    use std::{
        mem::{size_of, ManuallyDrop},
        ops::Range,
        slice::from_raw_parts,
        sync::Arc,
    };

    use crate::{base_trait::StringSequenceView, mutable::MutableStringSequence};

    /* ----------------------------------------- Common ----------------------------------------- */
    fn as_inner(slice: &[u8], pivot: usize) -> (&[u8], &[Range<u32>]) {
        let (index, buffer) = slice.split_at(pivot);
        // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
        let index = unsafe {
            from_raw_parts(
                index.as_ptr() as *const Range<u32>,
                index.len() / size_of::<Range<u32>>(),
            )
        };
        (buffer, index)
    }

    /* ------------------------------------- String Sequence ------------------------------------ */
    #[derive(Clone)]
    pub struct StringSequence {
        raw: Box<[u8]>,
        buffer_offset: usize,
    }

    impl_seq_view!(StringSequence);

    impl StringSequence {
        fn from_owned_index(index_buf: Vec<Range<u32>>, buffer: &[u8]) -> Self {
            let mut raw = {
                let mut raw_vec = ManuallyDrop::new(index_buf);
                let capacity = raw_vec.capacity() * size_of::<Range<u32>>() + buffer.len();
                let ptr = raw_vec.as_mut_ptr() as *mut u8;
                let length_u8 = raw_vec.len() * size_of::<Range<u32>>();

                // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
                unsafe { Vec::from_raw_parts(ptr, length_u8, capacity) }
            };

            let buffer_offset = raw.len();
            raw.reserve_exact(buffer.len());
            raw.extend_from_slice(buffer);
            raw.shrink_to_fit();

            Self { raw: raw.into_boxed_slice(), buffer_offset }
        }

        pub fn create_shared(&self) -> crate::view::SharedStringSequence {
            crate::view::SharedStringSequence::from(self)
        }
    }

    impl<'a, T: StringSequenceView> From<&'a T> for StringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            Self::from_owned_index(index.to_vec(), buffer)
        }
    }

    impl StringSequenceView for StringSequence {
        fn inner(&self) -> (&[u8], &[Range<u32>]) {
            as_inner(&self.raw, self.buffer_offset)
        }
    }

    impl From<MutableStringSequence> for StringSequence {
        fn from(value: MutableStringSequence) -> Self {
            value.build()
        }
    }

    impl MutableStringSequence {
        pub fn build(self) -> StringSequence {
            StringSequence::from_owned_index(self.index, &self.text)
        }
    }

    /* --------------------------------------- Shared Ref --------------------------------------- */

    /// Shared compact representation of a sequence of strings.
    #[derive(Clone)]
    pub struct SharedStringSequence {
        raw: Arc<[u8]>,
        buffer_offset: usize,
    }

    impl_seq_view!(SharedStringSequence);

    impl StringSequenceView for SharedStringSequence {
        fn inner(&self) -> (&[u8], &[Range<u32>]) {
            as_inner(&self.raw, self.buffer_offset)
        }
    }

    impl From<StringSequence> for SharedStringSequence {
        fn from(value: StringSequence) -> Self {
            Self { raw: value.raw.into(), buffer_offset: value.buffer_offset }
        }
    }

    impl<'a, T: StringSequenceView> From<&'a T> for SharedStringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            StringSequence::from_owned_index(index.to_vec(), buffer).into()
        }
    }

    impl From<MutableStringSequence> for SharedStringSequence {
        fn from(value: MutableStringSequence) -> Self {
            value.build().into()
        }
    }
}
pub mod mutable {
    use crate::base_trait::{up, StringSequenceView};

    /// A sequence of strings. This is used to represent a path.
    #[derive(Default)]
    pub struct MutableStringSequence {
        /// Internal buffer, to represent the sequence of strings.
        pub(crate) text: Vec<u8>,
        /// Offsets of the strings in the buffer.
        pub(crate) index: Vec<std::ops::Range<u32>>,
    }

    impl StringSequenceView for MutableStringSequence {
        fn inner(&self) -> (&[u8], &[std::ops::Range<u32>]) {
            (&self.text, &self.index)
        }
    }

    /* ---------------------------------------- Vwr Impl ---------------------------------------- */
    impl_seq_view!(MutableStringSequence);

    /* ---------------------------------------------------------------------------------------------- */
    /*                                          MANIPULATION                                          */
    /* ---------------------------------------------------------------------------------------------- */
    impl MutableStringSequence {
        /// Reserve space for internal string container.
        ///
        /// NOTE: Consider delimiter length when reserving space.
        pub fn reserve_buffer(&mut self, num_chars: usize) {
            self.text.reserve(num_chars);
        }

        /// Reserve space for internal index container. Index container indicates the number of
        /// `tokens` that can be appended without reallocation.
        pub fn reserve_index(&mut self, num_strings: usize) {
            self.index.reserve(num_strings);
        }

        /// Add list of references to the internal buffer.
        pub fn extend_from_slice<T: AsRef<str>>(&mut self, slice: &[T]) {
            let buffer_len = slice.iter().fold(0, |acc, s| acc + s.as_ref().len());
            self.reserve_buffer(buffer_len);
            self.reserve_index(slice.len());

            let mut offset = self.text.len();
            for s in slice {
                let s = s.as_ref();
                self.index.push(offset as _..(offset + s.len()) as _);
                self.text.extend_from_slice(s.as_bytes());
                offset = self.text.len();
            }
        }

        /// Append a string to the internal buffer. As we can't pre-calculate required space for
        /// text buffer, this is inherently inefficient compared to [`Self::extend_from_slice`].
        pub fn extend<T: AsRef<str>>(&mut self, into_iter: impl IntoIterator<Item = T>) {
            let mut iter = into_iter.into_iter();
            let num_elem_hint = iter.size_hint().0;

            // TODO:
        }

        /// Remove the string at the specified index.
        ///
        /// # Panics
        ///
        /// Panics if the index is out of bounds.
        pub fn remove(&mut self, index: usize) {
            let range = self.index.remove(index);
            self.index[index..].iter_mut().for_each(|x| {
                x.start -= range.len() as u32;
                x.end -= range.len() as u32;
            });
            self.text.drain(up(range));
        }

        /// Remove the last string quickly.
        pub fn pop_back(&mut self) {
            let range = self.index.pop().unwrap();
            self.text.drain(up(range));
        }

        /// Append a string to the end of the sequence.
        pub fn push_back(&mut self, value: &impl AsRef<str>) {
            let value = value.as_ref();
            let offset = self.text.len();
            self.index.push(offset as _..(offset + value.len()) as _);
            self.text.extend_from_slice(value.as_bytes());
        }

        /// Insert a string at the specified index.
        pub fn insert(&mut self, value: &impl AsRef<str>, index: usize) {
            let value = value.as_ref();
            let offset = self.text.len();
            self.index[index..].iter_mut().for_each(|x| {
                x.start += value.len() as u32;
                x.end += value.len() as u32;
            });
            self.index.insert(index, offset as _..(offset + value.len()) as _);
            self.text.extend_from_slice(value.as_bytes());
        }
    }

    /* ------------------------------------------------------------------------------------------ */
    /*                                            CTOR                                            */
    /* ------------------------------------------------------------------------------------------ */

    impl<'a, T: StringSequenceView> From<&'a T> for MutableStringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            Self { text: buffer.to_vec(), index: index.to_vec() }
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    //! Optional serde implementation for `StringSequence`.
}

pub use mutable::MutableStringSequence;
pub use view::{SharedStringSequence, StringSequence};
