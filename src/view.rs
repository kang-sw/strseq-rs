use std::{ops::Range, slice::from_raw_parts, sync::Arc};

use crate::base_trait::{up, StringSequenceView, ToRange};

/* ----------------------------------------- Common ----------------------------------------- */
#[inline]
fn as_inner(slice: &[[u32; 2]], text_start_index: usize) -> (&[u8], &[Range<u32>]) {
    let (index, buffer) = slice.split_at(text_start_index);

    // SAFETY: Plain POD conversion
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
    raw: Box<[[u32; 2]]>, // To keep the original alignment of the buffer
    index_count: usize,
}

impl_seq_view!(StringSequence);

impl StringSequence {
    /// Extends given index buffer by text buffer, to make compact representation of a sequence of
    /// strings. If `index_buf` reserved enough capacity, this function will not allocate.
    pub(crate) fn from_owned_index(index_buf: Vec<Range<u32>>, text: &[u8]) -> Self {
        // SAFETY: Conversion from `Vec<(u32, u32)>` to `Vec<u64>`:
        //  - Both are plain old data
        //    - Though `Range<u32>` doesn't define `Copy` trait, that is due to design decision, not
        //      because it has non-trivial logic for cloning.
        //  - Element size is the same
        //  - Destination type's memory alignment is more permissive
        let mut raw: Vec<[u32; 2]> = unsafe { std::mem::transmute(index_buf) };

        let text_start_index = raw.len();
        raw.reserve_exact((text.len() + 7) / 8);

        // SAFETY: It's just plain old data.
        let (beg, mid, end) = unsafe { text.align_to::<[u32; 2]>() };
        debug_assert!(beg.is_empty());

        raw.extend_from_slice(mid);

        let from_slice = |slice: &[u8]| {
            u32::from_ne_bytes(std::array::from_fn(|index| {
                slice.get(index).copied().unwrap_or_default()
            }))
        };

        // Push remaining bytes
        raw.push([from_slice(end), from_slice(&end[4.min(end.len())..])]);

        raw.shrink_to_fit();

        Self { raw: raw.into_boxed_slice(), index_count: text_start_index }
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
        as_inner(&self.raw, self.index_count)
    }
}

/* ------------------------------------------------------------------------------------------ */
/*                                      SHARED REFERENCE                                      */
/* ------------------------------------------------------------------------------------------ */

/// Shared compact representation of a sequence of strings.
#[derive(Clone)]
pub struct SharedStringSequence {
    raw: Arc<[[u32; 2]]>,
    index_count: usize,
    token_range: Range<u32>, // Naively expect we won't store more than 2^32 tokens.
}

impl_seq_view!(SharedStringSequence);

impl SharedStringSequence {
    pub fn subsequence(&self, range: impl ToRange) -> Self {
        let range = range.to_range(self.index_count);
        Self { token_range: range.start as _..range.end as _, ..self.clone() }
    }

    pub fn into_full_sequence(self) -> Self {
        Self { token_range: 0..self.index_count as _, ..self }
    }
}

impl StringSequenceView for SharedStringSequence {
    fn inner(&self) -> (&[u8], &[Range<u32>]) {
        let (text, index) = as_inner(&self.raw, self.index_count);
        (text, &index[up(self.token_range.clone())])
    }

    fn text(&self) -> &str {
        let (text, buffer) = self.inner();
        let start = buffer.first().map(|x| x.start).unwrap_or(0) as usize;
        let end = buffer.last().map(|x| x.end).unwrap_or(0) as usize;

        unsafe { std::str::from_utf8_unchecked(&text[start..end]) }
    }
}

impl From<StringSequence> for SharedStringSequence {
    fn from(value: StringSequence) -> Self {
        Self {
            raw: value.raw.into(),
            index_count: value.index_count,
            token_range: 0..value.index_count as _,
        }
    }
}

impl<'a, T: StringSequenceView> From<&'a T> for SharedStringSequence {
    fn from(value: &'a T) -> Self {
        let (buffer, index) = value.inner();
        StringSequence::from_owned_index(index.to_vec(), buffer).into()
    }
}
