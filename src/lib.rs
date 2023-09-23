
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
                std::hash::Hash::hash(self.full(), state)
            }
        }

        /* ---------------------------------------- Refs ---------------------------------------- */
        impl AsRef<str> for $Type {
            fn as_ref(&self) -> &str {
                self.full()
            }
        }

        impl AsRef<[u8]> for $Type {
            fn as_ref(&self) -> &[u8] {
                self.full().as_bytes()
            }
        }

        impl AsRef<std::path::Path> for $Type {
            fn as_ref(&self) -> &std::path::Path {
                std::path::Path::new(self.full())
            }
        }

        impl AsRef<std::ffi::OsStr> for $Type {
            fn as_ref(&self) -> &std::ffi::OsStr {
                std::ffi::OsStr::new(self.full())
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

            pub fn full(&self) -> &str {
                <Self as crate::base_trait::StringSequenceView>::full(self)
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

    /// Since there's no `impl Iterator` return for traits, we have to use this workaround.
    pub type StringSequenceIter<'a> = Map<
        Zip<Cloned<std::slice::Iter<'a, Range<usize>>>, Repeat<&'a [u8]>>,
        fn((Range<usize>, &'a [u8])) -> &'a str,
    >;

    /// Viewer functionality for `StringSequence`.
    pub(crate) trait StringSequenceView {
        fn inner(&self) -> (&[u8], &[Range<usize>]);

        fn iter(&self) -> StringSequenceIter {
            let (buffer, index) = self.inner();
            index
                .iter()
                .cloned()
                .zip(repeat(buffer))
                .map(move |(range, buffer)| 
                    // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
                    unsafe { std::str::from_utf8_unchecked(&buffer[range]) }
                )
        }

        fn slice(&self, range: impl ToRange) -> StringSequenceIter {
            let (buffer, index) = self.inner();
            let range = range.to_range(index.len());
            index[range]
                .iter()
                .cloned()
                .zip(repeat(buffer))
                .map(move |(range, buffer)| 
                    // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
                    unsafe { std::str::from_utf8_unchecked(&buffer[range]) }
                )
        }

        fn full(&self) -> &str {
            let (buffer, _) = self.inner();
            // SAFETY: Buffer is strictly managed to be valid UTF-8 string.
            unsafe { std::str::from_utf8_unchecked(buffer) }
        }

        fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.full())
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

    use crate::{base_trait::StringSequenceView, builder::StringSequenceBuilder};

    /* ----------------------------------------- Common ----------------------------------------- */
    fn as_inner(slice: &[u8], pivot: usize) -> (&[u8], &[Range<usize>]) {
        let (index, buffer) = slice.split_at(pivot);
        // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
        let index = unsafe {
            from_raw_parts(
                index.as_ptr() as *const Range<usize>,
                index.len() / size_of::<Range<usize>>(),
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
        fn from_owned_index(index_buf: Vec<Range<usize>>, buffer: &[u8]) -> Self {
            let mut raw_vec = ManuallyDrop::new(index_buf);
            let capacity = raw_vec.capacity() * size_of::<Range<usize>>() + buffer.len();
            let ptr = raw_vec.as_mut_ptr() as *mut u8;
            let length_u8 = raw_vec.len() * size_of::<Range<usize>>();

            // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
            let mut raw = unsafe { Vec::from_raw_parts(ptr, length_u8, capacity) };
            raw.reserve_exact(buffer.len());
            raw.extend_from_slice(buffer);
            raw.shrink_to_fit();

            Self {
                raw: raw.into_boxed_slice(),
                buffer_offset: length_u8,
            }
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
        fn inner(&self) -> (&[u8], &[Range<usize>]) {
            as_inner(&self.raw, self.buffer_offset)
        }
    }

    impl From<StringSequenceBuilder> for StringSequence {
        fn from(value: StringSequenceBuilder) -> Self {
            value.build()
        }
    }

    impl StringSequenceBuilder {
        pub fn build(self) -> StringSequence {
            StringSequence::from_owned_index(self.index, &self.buffer)
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
        fn inner(&self) -> (&[u8], &[Range<usize>]) {
            as_inner(&self.raw, self.buffer_offset)
        }
    }

    impl From<StringSequence> for SharedStringSequence {
        fn from(value: StringSequence) -> Self {
            Self {
                raw: value.raw.into(),
                buffer_offset: value.buffer_offset,
            }
        }
    }

    impl<'a, T: StringSequenceView> From<&'a T> for SharedStringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            StringSequence::from_owned_index(index.to_vec(), buffer).into()
        }
    }

    impl From<StringSequenceBuilder> for SharedStringSequence {
        fn from(value: StringSequenceBuilder) -> Self {
            value.build().into()
        }
    }
}
pub mod builder {
    use crate::base_trait::StringSequenceView;

    /// A sequence of strings. This is used to represent a path.
    #[derive(Default)]
    pub struct StringSequenceBuilder {
        /// Internal buffer, to represent the sequence of strings.
        pub(crate) buffer: Vec<u8>,
        /// Offsets of the strings in the buffer.
        pub(crate) index: Vec<std::ops::Range<usize>>,
    }

    impl StringSequenceView for StringSequenceBuilder {
        fn inner(&self) -> (&[u8], &[std::ops::Range<usize>]) {
            (&self.buffer, &self.index)
        }
    }

    /* ---------------------------------------- Vwr Impl ---------------------------------------- */
    impl_seq_view!(StringSequenceBuilder);

    /* ---------------------------------------------------------------------------------------------- */
    /*                                          MANIPULATION                                          */
    /* ---------------------------------------------------------------------------------------------- */
    impl StringSequenceBuilder {
        /// Reserve         
        pub fn reserve_buffer(&mut self, len_chars: usize) {
            self.buffer.reserve(len_chars);
        }

        pub fn reserve_index(&mut self, len_strings: usize) {
            self.index.reserve(len_strings);
        }

        pub fn extend_from_slice<T: AsRef<str>>(&mut self, slice: &[T], delim: &str) {
            let buffer_len = slice.iter().fold(0, |acc, s| acc + s.as_ref().len())
                + delim.len() * (slice.len().saturating_sub(1));
            self.reserve_buffer(buffer_len);
            self.reserve_index(slice.len());

            let mut offset = self.buffer.len();
            for s in slice {
                let s = s.as_ref();
                self.index.push(offset..offset + s.len());
                self.buffer.extend_from_slice(s.as_bytes());
                offset += s.len();
            }
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    //! Optional serde implementation for `StringSequence`.
}
