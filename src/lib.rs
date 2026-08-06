#![allow(clippy::match_overlapping_arm)]
use core::{borrow::Borrow, fmt::{self, Debug, Display}, hash::Hash, mem::size_of, num::NonZeroUsize, ops::Add, str};
use std::hash::Hasher;

const _: () = assert!(cfg!(any(target_pointer_width = "32", target_pointer_width = "64")));

#[cfg(feature = "usize")]
#[allow(non_camel_case_types)]
pub(crate) type ulen = usize;
#[cfg(not(feature = "usize"))]
#[allow(non_camel_case_types)]
pub(crate) type ulen = u32;

pub const MAX_SSO_LEN: usize = size_of::<usize>();

#[allow(unused)]
mod big_endian;
#[cfg(target_endian = "big")]
pub(crate) use big_endian as encoder;
#[allow(unused)]
mod little_endian;
#[cfg(target_endian = "little")]
pub(crate) use little_endian as encoder;

use crate::boxed_data::BoxedData;

pub(crate) mod boxed_data;
pub(crate) mod builder;
pub use builder::ArcStringBuilder;

#[repr(transparent)]
pub struct ArcString(NonZeroUsize);

impl ArcString {
	pub const fn empty() -> Self {
		Self(encoder::EMPTY)
	}

	pub const fn try_new_sso(s: &str) -> Option<Self> {
		if let Some(value) = encoder::try_encode_sso(s) {
			Some(Self(value))
		} else {
			None
		}
	}

	pub const fn can_be_sso(s: &str) -> bool {
		encoder::try_encode_sso(s).is_some()
	}

	pub(crate) fn get_boxed_data(&self) -> Option<BoxedData> {
		if let Some(ptr) = encoder::as_ptr(self.0.get()) {
			Some(BoxedData::from_ptr(ptr))
		} else {
			None
		}
	}

	pub(crate) fn try_take_boxed_data(self) -> Result<BoxedData, ArcString> {
		if let Some(boxed_data) = self.get_boxed_data() && unsafe {boxed_data.is_only_ref()} {
			core::mem::forget(self);
			Ok(boxed_data)
		} else {
			Err(self)
		}
	}

	pub fn from_iter<'a>(it: impl Iterator<Item = &'a str> + Clone) -> Self {
		ArcStringBuilder::from_iter(it).into_arcstring()
	}

	pub fn from_display<T: Display>(display: T) -> Self {
		ArcStringBuilder::from_display(display).into_arcstring()
	}

	pub fn as_str(&self) -> &str {
		encoder::decode(&self.0)
	}

	pub fn is_boxed(&self) -> bool {
		encoder::as_ptr(self.0.get()).is_some()
	}

	pub fn is_empty(&self) -> bool {
		encoder::decode(&self.0).is_empty()
	}

	pub fn len(&self) -> usize {
		encoder::decode(&self.0).len()
	}

	pub fn char_at(&self, idx: usize) -> Option<char> {
		self.as_str().get(idx..).and_then(|x| x.chars().next())
	}

	pub fn substr(&self, idx: usize, len: usize) -> Option<ArcString> {
		if idx == 0 && len == self.len() {
			Some(self.clone())
		} else {
			Some(self.as_str().get(idx..len)?.into())
		}
	}

	pub fn substr_from(&self, idx: usize) -> Option<ArcString> {
		if idx == 0 {
			Some(self.clone())
		} else {
			Some(self.as_str().get(idx..)?.into())
		}
	}
}

impl Default for ArcString {
	fn default() -> Self {
		Self::empty()
	}
}

impl Clone for ArcString {
	fn clone(&self) -> Self {
		if let Some(boxed_data) = self.get_boxed_data() {
			unsafe {boxed_data.increment_ref()};
		}
		Self(self.0)
	}
}

impl Drop for ArcString {
	fn drop(&mut self) {
		if let Some(boxed_data) = self.get_boxed_data() {
			unsafe {boxed_data.destroy_ref()};
		}
	}
}

impl PartialEq for ArcString {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 || self.as_str() == other.as_str()
	}
}

impl PartialEq<str> for ArcString {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<ArcStringBuilder> for ArcString {
	fn eq(&self, other: &ArcStringBuilder) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for ArcString {}

impl Hash for ArcString {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl Debug for ArcString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.as_str())
	}
}

impl Display for ArcString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl From<ArcStringBuilder> for ArcString {
	fn from(value: ArcStringBuilder) -> Self {
		value.into_arcstring()
	}
}

impl From<char> for ArcString {
	fn from(value: char) -> Self {
		value.encode_utf8(&mut [0; 4]).into()
	}
}

impl From<&str> for ArcString {
	fn from(s: &str) -> Self {
		if let Some(sso) = Self::try_new_sso(s) {
			sso
		} else {
			let boxed_data = BoxedData::alloc(s.len());
			unsafe {
				boxed_data.get_data_ptr().as_ptr().copy_from_nonoverlapping(s.as_ptr(), s.len());
				Self(encoder::encode_ptr(boxed_data.finalize(s.len() as ulen)))
			}
		}
	}
}

impl From<&mut str> for ArcString {
	fn from(value: &mut str) -> Self {
		Self::from(value as &str)
	}
}

impl From<String> for ArcString {
	fn from(value: String) -> Self {
		Self::from(value.as_str())
	}
}

impl Add for ArcString {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		let lhs_str = self.as_str();
		let rhs_str = rhs.as_str();
		if lhs_str.is_empty() {
			rhs
		} else if rhs_str.is_empty() {
			self
		} else {
			Self::from_iter([lhs_str, rhs_str].into_iter())
		}
	}
}

impl Add<&str> for ArcString {
	type Output = Self;
	fn add(self, rhs: &str) -> Self::Output {
		if rhs.is_empty() {
			self
		} else {
			Self::from_iter([self.as_str(), rhs].into_iter())
		}
	}
}

impl Add<char> for ArcString {
	type Output = Self;
	fn add(self, rhs: char) -> Self::Output {
		Self::from_iter([self.as_str(), rhs.encode_utf8(&mut [0; 4])].into_iter())
	}
}

impl Borrow<str> for ArcString {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for ArcString {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}
