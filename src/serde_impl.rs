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
