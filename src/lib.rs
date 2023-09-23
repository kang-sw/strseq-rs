/// TODO: In future, replace all `Range<u32>` usage to `[u32]`, since every tokens are adjacently
/// stored in memory, current implementation waste single word for each token to store duplicated
/// index offset!

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
                self.iter().for_each(|x| std::hash::Hash::hash(x, state))
            }
        }

        /* -------------------------------------- Comparing ------------------------------------- */
        impl<T: crate::base_trait::StringSequenceView> PartialEq<T> for $Type {
            fn eq(&self, other: &T) -> bool {
                self.iter().eq(other.iter())
            }
        }

        impl Eq for $Type {}

        impl<T: crate::base_trait::StringSequenceView> PartialOrd<T> for $Type {
            fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
                self.iter().partial_cmp(other.iter())
            }
        }

        impl Ord for $Type {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.iter().cmp(other.iter())
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
            fn tokens(&self) -> &[std::ops::Range<u32>] {
                let (_, index) = self.inner();
                index
            }

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

            pub fn first(&self) -> Option<&str> {
                self.get(0)
            }

            pub fn last(&self) -> Option<&str> {
                self.get(self.tokens().len().saturating_sub(1))
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
}
pub mod view {
    use std::{mem::ManuallyDrop, ops::Range, slice::from_raw_parts, sync::Arc};

    use crate::base_trait::StringSequenceView;

    /* ----------------------------------------- Common ----------------------------------------- */
    fn as_inner(slice: &[u64], text_start_index: usize) -> (&[u8], &[Range<u32>]) {
        let (index, buffer) = slice.split_at(text_start_index);
        // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
        unsafe {
            let index = from_raw_parts(index.as_ptr() as *const Range<u32>, index.len());
            let buffer_len = index.last().map(|x| x.end).unwrap_or(0) as usize;

            (from_raw_parts(buffer.as_ptr() as *const u8, buffer_len), index)
        }
    }

    /* ------------------------------------------------------------------------------------------ */
    /*                                   COMPACT REPRESENTATION                                   */
    /* ------------------------------------------------------------------------------------------ */

    #[derive(Clone)]
    pub struct StringSequence {
        raw: Box<[u64]>, // To keep the original alignment of the buffer
        text_start_index: usize,
    }

    impl_seq_view!(StringSequence);

    impl StringSequence {
        /// Provides memory reusing constructor.
        pub(crate) fn from_owned_index(index_buf: Vec<Range<u32>>, buffer: &[u8]) -> Self {
            let mut raw = {
                let mut raw_vec = ManuallyDrop::new(index_buf);
                let capacity = raw_vec.capacity();
                let ptr = raw_vec.as_mut_ptr() as *mut u64;
                let length_u8 = raw_vec.len();

                // SAFETY: We know that the index is a slice of ranges, which is a slice of usize.
                unsafe { Vec::from_raw_parts(ptr, length_u8, capacity) }
            };

            let text_start_index = raw.len();
            raw.reserve_exact((buffer.len() + 7) / 8);

            // SAFETY: It's just plain old data.
            let (beg, mid, end) = unsafe { buffer.align_to::<u64>() };
            debug_assert!(beg.is_empty());

            raw.extend_from_slice(mid);

            let mut last: [u8; 8] = [0; 8];
            last[..end.len()].copy_from_slice(end);
            raw.push(u64::from_ne_bytes(last));

            raw.shrink_to_fit();

            Self { raw: raw.into_boxed_slice(), text_start_index }
        }
    }

    /* --------------------------------------- Conversion --------------------------------------- */

    impl<'a, T: StringSequenceView> From<&'a T> for StringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            Self::from_owned_index(index.to_vec(), buffer)
        }
    }

    impl StringSequenceView for StringSequence {
        fn inner(&self) -> (&[u8], &[Range<u32>]) {
            as_inner(&self.raw, self.text_start_index)
        }
    }

    /* ------------------------------------------------------------------------------------------ */
    /*                                      SHARED REFERENCE                                      */
    /* ------------------------------------------------------------------------------------------ */

    /// Shared compact representation of a sequence of strings.
    #[derive(Clone)]
    pub struct SharedStringSequence {
        raw: Arc<[u64]>,
        text_start_index: usize,
    }

    impl_seq_view!(SharedStringSequence);

    impl StringSequenceView for SharedStringSequence {
        fn inner(&self) -> (&[u8], &[Range<u32>]) {
            as_inner(&self.raw, self.text_start_index)
        }
    }

    impl From<StringSequence> for SharedStringSequence {
        fn from(value: StringSequence) -> Self {
            Self { raw: value.raw.into(), text_start_index: value.text_start_index }
        }
    }

    impl<'a, T: StringSequenceView> From<&'a T> for SharedStringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            StringSequence::from_owned_index(index.to_vec(), buffer).into()
        }
    }
}
pub mod mutable {
    use crate::{
        base_trait::{retr, up, StringSequenceView, ToRange},
        view::SharedStringSequence,
        StringSequence,
    };

    /// A sequence of strings. This is used to represent a path.
    #[derive(Default, Clone)]
    pub struct MutableStringSequence {
        /// Internal buffer, to represent the sequence of strings.
        text: Vec<u8>,
        /// Offsets of the strings in the buffer.
        index: Vec<std::ops::Range<u32>>,
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
        /// Create a new empty sequence.
        pub fn new() -> Self {
            Self::default()
        }

        /// Create new instance with capacities
        pub fn with_capacity(num_tokens: usize, num_chars: usize) -> Self {
            Self { text: Vec::with_capacity(num_chars), index: Vec::with_capacity(num_tokens) }
        }

        /// Token array capacity
        pub fn token_capacity(&self) -> usize {
            self.index.capacity()
        }

        /// Text buffer capacity
        pub fn text_capacity(&self) -> usize {
            self.text.capacity()
        }

        /// Reserve space for internal string container.
        ///
        /// NOTE: Consider delimiter length when reserving space.
        pub fn reserve_buffer(&mut self, num_chars: usize) {
            self.text.reserve(num_chars);
        }

        /// Reserve space for internal index container. Index container indicates the number of
        /// `tokens` that can be appended without reallocation.
        pub fn reserve_index(&mut self, num_tokens: usize) {
            self.index.reserve(num_tokens);
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
            let iter = into_iter.into_iter();
            let num_elem_hint = iter.size_hint().0;

            self.reserve_index(num_elem_hint);
            iter.for_each(|s| self.push_back(&s));
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
        pub fn push_back(&mut self, value: impl AsRef<str>) {
            let value = value.as_ref();
            let offset = self.text.len();
            self.index.push(offset as _..(offset + value.len()) as _);
            self.text.extend_from_slice(value.as_bytes());
        }

        /// Insert a string at the specified index.
        pub fn insert(&mut self, index: usize, value: impl AsRef<str>) {
            let value = value.as_ref().as_bytes();
            let insert_offset =
                self.index.get(index).map(|x| x.start).unwrap_or(self.text.len() as _);
            let offset = insert_offset as usize;

            self.index[index..].iter_mut().for_each(|x| {
                x.start += value.len() as u32;
                x.end += value.len() as u32;
            });
            self.index.insert(index, offset as _..(offset + value.len()) as _);

            self.text.splice(offset..offset, value.iter().copied());
        }

        pub fn clear(&mut self) {
            self.text.clear();
            self.index.clear();
        }

        pub fn drain(&mut self, range: impl ToRange) -> Drain {
            let self_ptr = self as *mut _;

            let range = range.to_range(self.index.len());

            if range.is_empty() {
                // Early return if the range is empty
                return Drain {
                    inner: self_ptr,
                    iter: self.index.drain(0..0),
                    text_erase_range: 0..0,
                };
            }

            let begin = self.index[range.start].start;
            let end = self.index[range.end - 1].end;

            // Subtract later element's offset before we process draining
            let removed_text_len = end - begin;
            self.index[range.end..].iter_mut().for_each(|x| {
                x.start -= removed_text_len;
                x.end -= removed_text_len;
            });

            let drain_iter = self.index.drain(range);
            Drain { inner: self_ptr, iter: drain_iter, text_erase_range: begin..end }
        }

        pub fn into_string_sequence(self) -> StringSequence {
            StringSequence::from_owned_index(self.index, &self.text)
        }
    }

    /* ------------------------------------- Drain Iterator ------------------------------------- */

    pub struct Drain<'a> {
        inner: *mut MutableStringSequence,
        text_erase_range: std::ops::Range<u32>,
        iter: std::vec::Drain<'a, std::ops::Range<u32>>,
    }

    impl<'a> Drop for Drain<'a> {
        fn drop(&mut self) {
            // SAFETY: We won't touch the `self.index` here, which is mutably borrowed for `iter`
            unsafe { &mut *self.inner }.text.drain(up(self.text_erase_range.clone()));
        }
    }

    impl<'a> Iterator for Drain<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            // SAFETY: We won't touch the `self.index` here, which is mutably borrowed for `iter`
            self.iter.next().map(|range| unsafe { retr(&(*self.inner).text, range) })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }
    }

    /* ------------------------------------------------------------------------------------------ */
    /*                                         CONVERSION                                         */
    /* ------------------------------------------------------------------------------------------ */

    impl<T: AsRef<str>> FromIterator<T> for MutableStringSequence {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut this = Self::default();
            this.extend(iter);
            this
        }
    }

    impl MutableStringSequence {
        pub fn from_slice(slice: &[impl AsRef<str>]) -> Self {
            let mut this = Self::default();
            this.extend_from_slice(slice);
            this
        }
    }

    impl From<String> for MutableStringSequence {
        fn from(value: String) -> Self {
            let unique_index = 0..value.len() as u32;
            Self { index: [unique_index].into(), text: value.into_bytes() }
        }
    }

    impl From<MutableStringSequence> for String {
        fn from(value: MutableStringSequence) -> Self {
            // SAFETY: We know that the buffer is strictly managed to be valid UTF-8 string.
            unsafe { String::from_utf8_unchecked(value.text) }
        }
    }

    impl<'a, T: StringSequenceView> From<&'a T> for MutableStringSequence {
        fn from(value: &'a T) -> Self {
            let (buffer, index) = value.inner();
            Self { text: buffer.to_vec(), index: index.to_vec() }
        }
    }

    impl From<StringSequence> for MutableStringSequence {
        fn from(value: StringSequence) -> Self {
            let (buffer, index) = value.inner();
            Self { text: buffer.to_vec(), index: index.to_vec() }
        }
    }

    impl From<MutableStringSequence> for StringSequence {
        fn from(value: MutableStringSequence) -> Self {
            Self::from_owned_index(value.index, &value.text)
        }
    }

    impl From<MutableStringSequence> for SharedStringSequence {
        fn from(value: MutableStringSequence) -> Self {
            Self::from(StringSequence::from(value))
        }
    }

    impl From<SharedStringSequence> for MutableStringSequence {
        fn from(value: SharedStringSequence) -> Self {
            Self::from(&value)
        }
    }

    impl<'a, T: AsRef<str> + 'a> FromIterator<T> for StringSequence {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            MutableStringSequence::from_iter(iter).into()
        }
    }

    impl StringSequence {
        pub fn from_slice(slice: &[impl AsRef<str>]) -> Self {
            MutableStringSequence::from_slice(slice).into()
        }
    }

    impl<'a, T: AsRef<str> + 'a> FromIterator<T> for SharedStringSequence {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            MutableStringSequence::from_iter(iter).into()
        }
    }

    impl SharedStringSequence {
        pub fn from_slice(slice: &[impl AsRef<str>]) -> Self {
            MutableStringSequence::from_slice(slice).into()
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    //! Optional serde implementation

    /* ------------------------------------------------------------------------------------------ */
    /*                                 BORROWER FOR SERIALIZATION                                 */
    /* ------------------------------------------------------------------------------------------ */

    struct Borrower<'a>(&'a [u8], &'a [Range<u32>]);

    use std::ops::Range;

    use serde::{de::SeqAccess, ser::SerializeSeq, Deserializer, Serialize, Serializer};

    use crate::{
        base_trait::StringSequenceView, mutable::MutableStringSequence, SharedStringSequence,
        StringSequence,
    };

    impl<'a> serde::Serialize for Borrower<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let iter = self.iter();
            let mut seq = serializer.serialize_seq(Some(iter.len()))?;

            for str in iter {
                seq.serialize_element(str)?;
            }

            seq.end()
        }
    }

    impl<'a> crate::base_trait::StringSequenceView for Borrower<'a> {
        fn inner(&self) -> (&[u8], &[Range<u32>]) {
            (self.0, self.1)
        }
    }

    /* ------------------------------------- Serialize Impls ------------------------------------ */

    macro_rules! gen_ser {
        ($type_name:path) => {
            impl<'a> Serialize for $type_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let (a, b) = self.inner();
                    Borrower(a, b).serialize(serializer)
                }
            }
        };
    }

    gen_ser!(crate::view::StringSequence);
    gen_ser!(crate::view::SharedStringSequence);
    gen_ser!(crate::mutable::MutableStringSequence);

    /* ------------------------------------------------------------------------------------------ */
    /*                                       DESERIALIZATION                                      */
    /* ------------------------------------------------------------------------------------------ */

    // `MutableStringSequence` -> `StringSequence` or `SharedStringSequence`

    impl<'de> serde::de::Deserialize<'de> for MutableStringSequence {
        fn deserialize_in_place<'a, D>(deserializer: D, place: &'a mut Self) -> Result<(), D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visitor<'a>(&'a mut MutableStringSequence);

            impl<'a, 'de> serde::de::Visitor<'de> for Visitor<'a> {
                type Value = ();

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a sequence of strings")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    self.0.clear();

                    if let Some(size) = seq.size_hint() {
                        self.0.reserve_index(size);
                    }

                    while let Some(value) = seq.next_element::<&str>()? {
                        self.0.push_back(value);
                    }

                    Ok(())
                }
            }

            deserializer.deserialize_seq(Visitor(place))
        }

        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let mut seq = Self::default();
            Self::deserialize_in_place(deserializer, &mut seq)?;
            Ok(seq)
        }
    }

    impl<'de> serde::de::Deserialize<'de> for StringSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            MutableStringSequence::deserialize(deserializer).map(Into::into)
        }
    }

    impl<'de> serde::de::Deserialize<'de> for SharedStringSequence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            MutableStringSequence::deserialize(deserializer).map(Into::into)
        }
    }
}

pub use mutable::MutableStringSequence;
pub use view::{SharedStringSequence, StringSequence};

#[cfg(test)]
mod tests {
    use crate::{base_trait::ToRange, MutableStringSequence, SharedStringSequence, StringSequence};

    #[test]
    fn basics() {
        let hello_world = ["Hello,", " World!", "asd", " ahgteaw", "adsgads", "dsagkd"];

        assert_eq!(
            dbg!(MutableStringSequence::from_iter(hello_world)),
            StringSequence::from_iter(hello_world)
        );

        assert_eq!(
            MutableStringSequence::from_iter(hello_world),
            SharedStringSequence::from_iter(hello_world)
        );
    }

    #[test]
    fn mutation() {
        let mut seq = MutableStringSequence::new();

        seq.push_back("hello");
        assert!(seq.iter().eq(["hello"]));
        assert!(seq.clone().into_string_sequence().iter().eq(["hello"]));
        assert_eq!(seq.text(), "hello");

        seq.push_back("world");
        assert!(seq.iter().eq(["hello", "world"]));
        assert!(seq.clone().into_string_sequence().iter().eq(["hello", "world"]));
        assert_eq!(seq.text(), "helloworld");

        seq.insert(0, "!");
        assert!(dbg!(&seq).iter().eq(["!", "hello", "world"]));
        assert!(dbg!(&seq.clone().into_string_sequence()).iter().eq(["!", "hello", "world"]));
        assert_eq!(seq.text(), "!helloworld");

        seq.insert(0, "howdy");
        assert!(seq.iter().eq(["howdy", "!", "hello", "world"]));
        assert!(seq.clone().into_string_sequence().iter().eq(["howdy", "!", "hello", "world"]));
        assert_eq!(seq.text(), "howdy!helloworld");

        assert!(seq.drain(1..3).eq(["!", "hello"]));
        assert!(seq.iter().eq(["howdy", "world"]));
        assert!(seq.clone().into_string_sequence().iter().eq(["howdy", "world"]));
        assert_eq!(seq.text(), "howdyworld");

        assert_eq!(seq.drain(0..0).count(), 0);
        assert!(seq.iter().eq(["howdy", "world"]));

        seq.extend(seq.clone().drain(..).chain(seq.clone().drain(..)));
        assert!(seq.iter().eq(["howdy", "world", "howdy", "world", "howdy", "world"]));
    }

    macro_rules! generate_view_test {
        ($func_name:ident, $type_name:ty) => {
            fn $func_name(view: $type_name, expected: &[&str]) {
                assert!(view.iter().eq(expected.iter().copied()));
                assert_eq!(view.len(), expected.len());
                assert_eq!(view.text(), expected.join(""));
                assert_eq!(view.first(), expected.first().copied());
                assert_eq!(view.last(), expected.last().copied());

                let array_len = expected.len();
                let ranges = [
                    ToRange::to_range(.., array_len),
                    ToRange::to_range(..0, array_len),
                    ToRange::to_range(0.., array_len),
                    ToRange::to_range(..array_len, array_len),
                    ToRange::to_range(0..array_len, array_len),
                    ToRange::to_range(0..array_len / 2, array_len),
                    ToRange::to_range(array_len / 2..array_len, array_len),
                ];

                assert!(view.starts_with(&expected[..]));
                assert!(view.starts_with(&expected[..array_len / 2]));
                assert!(view.ends_with(&expected[array_len / 2..array_len]));

                for range in ranges.iter() {
                    assert!(view.slice(range.clone()).eq(expected[range.clone()].iter().copied()));
                    assert!(view.contains(&expected[range.clone()]));
                }
            }
        };
    }

    generate_view_test!(test_view_seq, StringSequence);
    generate_view_test!(test_view_mut, MutableStringSequence);
    generate_view_test!(test_view_share, SharedStringSequence);

    #[test]
    fn view() {
        let vars: &[&[&str]] = &[
            &["dsagdsaf", "ewarsdag", "adsgsdag", "dfd0k99", "llas0px;;;"], // only ascii
            &["dsagdsaf", "ewarsdag", "adsgsdag", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"], // with unicode
            &["dsagdsaf", "ashg", "asdglkjic090a", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"],
            &["ㅐㄴㅇ0", "ㄹㅇㄴ02.,", " ㅇㄴ마🤣🤣🤣", "ㅇㄴㅁ000"], // with emoji (4B)
            &["asdlk0f99"],
            &[],
            &["--9dsc0", "0as-=-ㄴㅁ0", "ㅊ,ㅍ0009", "ㄴ00ㅏㅏ0-ㅔ;"],
        ];

        for var in vars {
            let view = MutableStringSequence::from_slice(var);
            test_view_seq(view.clone().into(), var);
            test_view_share(view.clone().into(), var);
            test_view_mut(view, var);
        }

        let var = ["dsagdsaf", "ewarsdag", "adsgsdag", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"];
        let ser = serde_json::to_string(&var).unwrap();
        let de: MutableStringSequence = serde_json::from_str(&ser).unwrap();
        let de_ser = serde_json::to_string(&de).unwrap();
        assert_eq!(ser, de_ser);

        test_view_seq(de.clone().into(), &var);
        test_view_share(de.clone().into(), &var);
        test_view_mut(de, &var);
    }

    #[test]
    fn stability() {
        for _ in 0..5 {
            let var: Vec<_> = (0..5000)
                .map(|_| {
                    String::from_iter((0..rand::random::<u8>()).map(|_| rand::random::<char>()))
                })
                .collect();

            let var = Vec::from_iter(var.iter().map(|x| x.as_str()));
            let var = &var[..];
            let view = MutableStringSequence::from_slice(&var);
            test_view_seq(view.clone().into(), var);
            test_view_share(view.clone().into(), var);
            test_view_mut(view, var);
        }
    }
}
