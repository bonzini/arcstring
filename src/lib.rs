#![allow(clippy::match_overlapping_arm)]
use core::{borrow::Borrow, cmp::Ordering, convert::Infallible, ptr::NonNull, fmt::{self, Debug, Display}, hash::Hash, mem::size_of, ops::{Add, Deref}, str::{self, FromStr, Utf8Error}};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::hash::Hasher;
use std::path::Path;

const _: () = assert!(cfg!(any(target_pointer_width = "32", target_pointer_width = "64")));

#[cfg(feature = "usize")]
#[allow(non_camel_case_types)]
pub(crate) type ulen = usize;
#[cfg(not(feature = "usize"))]
#[allow(non_camel_case_types)]
pub(crate) type ulen = u32;

pub const MAX_SSO_LEN: usize = size_of::<usize>();

pub(crate) mod encoder;

use crate::boxed_data::{BoxedData, Header};
pub use boxed_data::StaticHeader;

pub(crate) mod boxed_data;
pub(crate) mod builder;
pub use builder::ArcStringBuilder;

#[cfg(feature = "serde")]
mod serde;

#[repr(transparent)]
pub struct ArcString(NonNull<Header>);

/* the string data is immutable and the reference count is atomic, so a header
   can be shared and handed over between threads */
unsafe impl Send for ArcString {}
unsafe impl Sync for ArcString {}

/// Creates an [`ArcString`] from a string constant without allocating: short
/// strings are stored inline, longer ones point to a descriptor that constant
/// promotion places in static memory.
#[macro_export]
macro_rules! arcstring {
	($s:expr) => {{
		const SSO: Option<$crate::ArcString> = $crate::ArcString::try_new_sso($s);
		const DESC: &$crate::StaticHeader<{$s.len()}> = &$crate::StaticHeader::new($s);
		match SSO {
			Some(sso) => sso,
			None => unsafe {$crate::ArcString::from_static_raw(DESC)}
		}
	}};
}

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

	/// Creates an `ArcString` that points to the static descriptor `header`,
	/// without allocating and without reference counting. Normally the descriptor
	/// is promoted to static memory by the [`arcstring!`] macro.
	#[inline(always)]
	pub fn from_static_raw<const N: usize>(header: &'static StaticHeader<N>) -> Self {
		let ptr = NonNull::from(header).cast::<Header>();
		debug_assert!((ptr.addr().get() & 7) == 0);
		Self(encoder::encode_literal(ptr))
	}

	/// Creates an `ArcString` that leaks the contents of `s`, so that the result
	/// behaves like a string literal: it is not reference counted and it is never
	/// freed. Strings that fit inline leak nothing.
	pub fn leak_from(s: impl Into<ArcStringBuilder>) -> Self {
		s.into().leak()
	}

	pub(crate) fn get_boxed_data(&self) -> Option<BoxedData<'_>> {
		if let Some(ptr) = encoder::as_ptr(self.0) {
			Some(unsafe { BoxedData::from_ptr(ptr) })
		} else {
			None
		}
	}

	pub(crate) fn try_take_boxed_data(self) -> Result<NonNull<Header>, ArcString> {
		if let Some(boxed_data) = self.get_boxed_data() && boxed_data.is_only_ref() {
			let header = boxed_data.into_inner();
			core::mem::forget(self);
			Ok(header)
		} else {
			Err(self)
		}
	}

	pub fn from_display<T: Display>(display: T) -> Self {
		ArcStringBuilder::from_display(display).into_arcstring()
	}

	pub fn as_str(&self) -> &str {
		encoder::decode(&self.0)
	}

	pub fn is_boxed(&self) -> bool {
		encoder::as_ptr(self.0).is_some()
	}

	pub fn is_empty(&self) -> bool {
		encoder::decode(&self.0).is_empty()
	}

	pub fn len(&self) -> usize {
		encoder::decode(&self.0).len()
	}

	pub fn as_static(&self) -> Option<&'static str> {
		encoder::as_static(&self.0)
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
		if let Some(mut boxed_data) = self.get_boxed_data() {
			boxed_data.increment_ref();
		}
		Self(self.0)
	}
}

impl Drop for ArcString {
	fn drop(&mut self) {
		if let Some(boxed_data) = self.get_boxed_data() {
			boxed_data.destroy_ref();
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

impl PartialEq<&str> for ArcString {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<ArcString> for str {
	fn eq(&self, other: &ArcString) -> bool {
		self == other.as_str()
	}
}

impl PartialEq<ArcString> for &str {
	fn eq(&self, other: &ArcString) -> bool {
		*self == other.as_str()
	}
}

impl PartialEq<ArcStringBuilder> for ArcString {
	fn eq(&self, other: &ArcStringBuilder) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for ArcString {}

impl PartialOrd for ArcString {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for ArcString {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

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
		ArcStringBuilder::from(s).into_arcstring()
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

impl From<&String> for ArcString {
	fn from(value: &String) -> Self {
		Self::from(value.as_str())
	}
}

impl From<Box<str>> for ArcString {
	fn from(value: Box<str>) -> Self {
		Self::from(&*value)
	}
}

impl From<Cow<'_, str>> for ArcString {
	fn from(value: Cow<'_, str>) -> Self {
		Self::from(&*value)
	}
}

impl TryFrom<&[u8]> for ArcString {
	type Error = Utf8Error;
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		Ok(Self::from(str::from_utf8(value)?))
	}
}

impl TryFrom<Vec<u8>> for ArcString {
	type Error = Utf8Error;
	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Ok(Self::from(str::from_utf8(&value)?))
	}
}

impl FromStr for ArcString {
	type Err = Infallible;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(s))
	}
}

/* the concatenation itself is the builder's job, so anything it can collect
   can be collected into an ArcString too */
impl<T> FromIterator<T> for ArcString where ArcStringBuilder: FromIterator<T> {
	fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
		ArcStringBuilder::from_iter(it).into_arcstring()
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
			Self::from_iter([lhs_str, rhs_str])
		}
	}
}

impl Add<&str> for ArcString {
	type Output = Self;
	fn add(self, rhs: &str) -> Self::Output {
		if rhs.is_empty() {
			self
		} else {
			Self::from_iter([self.as_str(), rhs])
		}
	}
}

impl Add<char> for ArcString {
	type Output = Self;
	fn add(self, rhs: char) -> Self::Output {
		Self::from_iter([self.as_str(), rhs.encode_utf8(&mut [0; 4])])
	}
}

impl Borrow<str> for ArcString {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

/* this is what makes the whole of str, and indexing by a range, available on
   an ArcString without going through as_str() */
impl Deref for ArcString {
	type Target = str;
	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for ArcString {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for ArcString {
	fn as_ref(&self) -> &[u8] {
		self.as_str().as_bytes()
	}
}

impl AsRef<OsStr> for ArcString {
	fn as_ref(&self) -> &OsStr {
		OsStr::new(self.as_str())
	}
}

impl AsRef<Path> for ArcString {
	fn as_ref(&self) -> &Path {
		Path::new(self.as_str())
	}
}
