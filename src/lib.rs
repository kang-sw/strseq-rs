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
mod base_trait;
pub mod mutable;
pub mod view;

#[cfg(feature = "serde")]
mod serde_impl;

#[cfg(test)]
mod tests;

pub use mutable::MutableStringSequence;
pub use view::{SharedStringSequence, StringSequence};
