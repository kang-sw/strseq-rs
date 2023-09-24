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
    /// Extends given index buffer by text buffer, to make compact representation of a sequence of
    /// strings. If `index_buf` reserved enough capacity, this function will not allocate.
    pub(crate) fn from_owned_index(index_buf: Vec<Range<u32>>, text: &[u8]) -> Self {
        // SAFETY: Conversion from `Vec<(u32, u32)>` to `Vec<u64>`:
        //  - Both are plain old data
        //    - Though `Range<u32>` doesn't define `Copy` trait, that is due to design decision, not
        //      because it has non-trivial logic for cloning.
        //  - Element size is the same
        //  - Destination type's memory alignment is more permissive
        let mut raw: Vec<u64> = unsafe { std::mem::transmute(index_buf) };

        let text_start_index = raw.len();
        raw.reserve_exact((text.len() + 7) / 8);

        // SAFETY: It's just plain old data.
        let (beg, mid, end) = unsafe { text.align_to::<u64>() };
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
    token_range: Range<u32>, // Naively expect we won't store more than 2^32 tokens.
}

impl_seq_view!(SharedStringSequence);

impl StringSequenceView for SharedStringSequence {
    fn inner(&self) -> (&[u8], &[Range<u32>]) {
        as_inner(&self.raw, self.text_start_index)
    }
}

impl From<StringSequence> for SharedStringSequence {
    fn from(value: StringSequence) -> Self {
        Self {
            raw: value.raw.into(),
            text_start_index: value.text_start_index,
            token_range: 0..value.text_start_index as _,
        }
    }
}

impl<'a, T: StringSequenceView> From<&'a T> for SharedStringSequence {
    fn from(value: &'a T) -> Self {
        let (buffer, index) = value.inner();
        StringSequence::from_owned_index(index.to_vec(), buffer).into()
    }
}
