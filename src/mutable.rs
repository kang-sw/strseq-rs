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
        let insert_offset = self.index.get(index).map(|x| x.start).unwrap_or(self.text.len() as _);
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
            return Drain { inner: self_ptr, iter: self.index.drain(0..0), text_erase_range: 0..0 };
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
