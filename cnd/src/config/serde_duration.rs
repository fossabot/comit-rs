use serde::{de, export::fmt, Deserializer, Serialize, Serializer};
use std::time::Duration;

pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a duration in seconds")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs(value))
        }
    }

    deserializer.deserialize_u64(Visitor)
}

// reference: serde_url crate.

/// Serializes `value` with a given serializer.
// We need this in order to use `#[serde(with = "super::serde_duration")]`
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    for<'a> Ser<'a, T>: Serialize,
{
    Ser::new(value).serialize(serializer)
}

// A wrapper so we can implement custom serialize of inner type.
#[derive(Debug)]
pub struct Ser<'a, T>(&'a T);

impl<'a, T> Ser<'a, T>
where
    Ser<'a, T>: Serialize,
{
    /// Returns a new `Ser` wrapper.
    #[inline(always)]
    pub fn new(value: &'a T) -> Self {
        Ser(value)
    }
}

/// Serializes this URL into a `serde` stream.
impl<'a> Serialize for Ser<'a, Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.as_secs().to_string())
    }
}
