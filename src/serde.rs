use core::fmt;

use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::ArcString;

impl Serialize for ArcString {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}

struct ArcStringVisitor;

impl Visitor<'_> for ArcStringVisitor {
	type Value = ArcString;

	fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("a string")
	}

	fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok(ArcString::from(v))
	}

	fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
		Ok(ArcString::from(v))
	}

	/* the data is copied into the ArcString anyway, so byte input can be
	   accepted as long as it is valid UTF-8 */
	fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
		ArcString::try_from(v).map_err(|_| E::invalid_value(Unexpected::Bytes(v), &self))
	}
}

impl<'de> Deserialize<'de> for ArcString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(ArcStringVisitor)
	}
}
