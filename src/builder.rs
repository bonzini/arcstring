use core::fmt::{self, Display, Write};
use core::ptr::NonNull;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Add;

use crate::{ArcString, MAX_SSO_LEN, boxed_data::{BoxedData, Header}, encoder, ulen};

pub struct ArcStringBuilder {
	capacity: ulen,
	length: ulen,
	/* either a pointer to the Header of the string being built, or the string
	   itself stored in place of an address, padded with 0xFF bytes exactly like
	   an SSO ArcString: everything past self.length has to stay 0xFF, so that the
	   word can be handed over to an ArcString without being re-encoded */
	data: *mut Header
}

/* the buffer is owned by the builder alone, so it can be shared and handed over
   between threads as freely as the string it is building */
unsafe impl Send for ArcStringBuilder {}
unsafe impl Sync for ArcStringBuilder {}

impl ArcStringBuilder {
	pub const fn new() -> Self {
		Self {
			capacity: MAX_SSO_LEN as ulen,
			length: 0,
			data: encoder::EMPTY.as_ptr()
		}
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let mut builder = Self::new();
		builder.reserve(capacity);
		builder
	}

	pub fn from_iter<'a>(it: impl Iterator<Item = &'a str> + Clone) -> Self {
		let mut len = 0;
		for s in it.clone() {
			len += s.len();
		}
		let mut builder = Self::with_capacity(len);
		for s in it {
			builder.push_str(s);
		}
		builder
	}

	pub const fn can_be_sso(s: &str) -> bool {
		s.len() <= MAX_SSO_LEN
	}

	pub const fn try_new_sso(s: &str) -> Option<Self> {
		if Self::can_be_sso(s) {
			let mut data = [!0; MAX_SSO_LEN];
			let mut i = 0;
			while i < s.len() {
				data[i] = s.as_bytes()[i];
				i += 1;
			}
			Some(Self {
				capacity: MAX_SSO_LEN as ulen,
				length: s.len() as ulen,
				/* an empty string, and only that, encodes to a null pointer */
				data: if let Some(data) = encoder::encode_inline(data) {
					data.as_ptr()
				} else {
					core::ptr::null_mut()
				}
			})
		} else {
			None
		}
	}

	pub fn from_display<T: Display>(display: T) -> Self {
		let mut builder = Self::new();
		write!(&mut builder, "{display}").unwrap();
		builder
	}

	fn get_boxed_data(&self) -> Option<BoxedData> {
		if self.capacity as usize == MAX_SSO_LEN {
			None
		} else {
			Some(unsafe {BoxedData::from_ptr(NonNull::new_unchecked(self.data))})
		}
	}

	fn get_data_ptr(&self) -> NonNull<u8> {
		if let Some(boxed_data) = self.get_boxed_data() {
			boxed_data.get_data_ptr()
		} else {
			unsafe {
				NonNull::new_unchecked((&raw const self.data).cast_mut()).cast()
			}
		}
	}

	/* an inline string is written through the builder itself, so the pointer has
	   to be derived from a mutable borrow rather than from a shared one */
	fn get_data_ptr_mut(&mut self) -> NonNull<u8> {
		if let Some(boxed_data) = self.get_boxed_data() {
			boxed_data.get_data_ptr()
		} else {
			unsafe {
				NonNull::new_unchecked(&raw mut self.data).cast()
			}
		}
	}

	pub fn capacity(&self) -> usize {
		self.capacity as usize
	}

	pub fn len(&self) -> usize {
		self.length as usize
	}

	pub fn is_empty(&self) -> bool {
		self.length == 0
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			str::from_utf8_unchecked(core::ptr::slice_from_raw_parts(self.get_data_ptr().as_ptr(), self.length as usize).as_ref_unchecked())
		}
	}

	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe {
			str::from_utf8_unchecked_mut(core::ptr::slice_from_raw_parts_mut(self.get_data_ptr_mut().as_ptr(), self.length as usize).as_mut_unchecked())
		}
	}

	fn set_capacity_internal(&mut self, new_capacity: usize) {
		assert!(new_capacity > MAX_SSO_LEN);
		if let Some(boxed_data) = self.get_boxed_data() {
			unsafe {
				self.data = boxed_data.realloc(self.capacity as usize, new_capacity).into_inner().as_ptr();
			}
		} else {
			let boxed_data = BoxedData::alloc(new_capacity);
			unsafe {
				boxed_data.get_data_ptr().copy_from_nonoverlapping(self.get_data_ptr(), self.length as usize);
			}
			self.data = boxed_data.into_inner().as_ptr();
		}
		self.capacity = new_capacity.try_into().expect("ArcStringBuilder grew too long for the length type being used");
	}

	pub fn reserve_exact(&mut self, extra: usize) {
		let new_capacity = self.length as usize + extra;
		if new_capacity > self.capacity as usize {
			self.set_capacity_internal(new_capacity);
		}
	}

	pub fn reserve(&mut self, extra: usize) {
		let new_capacity = self.length as usize + extra;
		if new_capacity > self.capacity as usize {
			self.set_capacity_internal(new_capacity.next_power_of_two());
		}
	}

	pub fn shrink_to_fit(&mut self) {
		if self.capacity as usize > MAX_SSO_LEN && self.capacity > self.length {
			if let Some(inline) = Self::try_new_sso(self.as_str()) {
                                // self.length must be <= MAX_SSO_LEN, so the string fits inline
                                // again.  the assignment drops the old builder and the buffer with it.
				*self = inline;
			} else {
				self.set_capacity_internal(self.length as usize);
			}
		}
	}

	pub fn push(&mut self, c: char) {
		self.push_str(c.encode_utf8(&mut [0; 4]));
	}

	pub fn push_str(&mut self, s: &str) {
		self.reserve(s.len());
		unsafe {
			self.get_data_ptr_mut().byte_add(self.length as usize).as_ptr().copy_from_nonoverlapping(s.as_ptr(), s.len());
		}
		self.length = (self.length as usize + s.len()) as ulen;
	}

	#[inline]
	fn into_boxed_data(self) -> NonNull<Header> {
		let boxed_data = if let Some(boxed_data) = self.get_boxed_data() {
			if self.capacity > self.length {
				unsafe {boxed_data.realloc(self.capacity as usize, self.length as usize)}
			} else {
				boxed_data
			}
		} else {
			let self_str = self.as_str();
			unsafe {
				let boxed_data = BoxedData::alloc(self.length as usize);
				boxed_data.get_data_ptr().as_ptr().copy_from_nonoverlapping(self_str.as_bytes().as_ptr(), self_str.len());
				boxed_data
			}
		};
		let header = unsafe {boxed_data.finalize(self.length)};
		core::mem::forget(self);
		header
	}

	fn try_into_arcstring_sso(&self) -> Option<NonNull<Header>> {
		// an inline builder already holds the SSO encoding of its contents, so the
		// word only has to be moved over (if it is nonzero)
		if self.capacity as usize == MAX_SSO_LEN {
			NonNull::new(self.data)
		} else {
			encoder::try_encode_sso(self.as_str())
		}
	}

	pub fn into_arcstring(self) -> ArcString {
		ArcString(self.try_into_arcstring_sso().unwrap_or_else(|| encoder::encode_ptr(self.into_boxed_data())))
	}

	/// Converts into an `ArcString` that leaks the buffer of the builder, so that
	/// the result behaves like a string literal: it is not reference counted and
	/// it is never freed. Strings that fit inline leak nothing.
	pub fn leak(self) -> ArcString {
		ArcString(self.try_into_arcstring_sso().unwrap_or_else(|| encoder::encode_literal(self.into_boxed_data())))
	}

}

impl Default for ArcStringBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for ArcStringBuilder {
	fn clone(&self) -> Self {
		if let Some(boxed_data) = self.get_boxed_data() {
			unsafe {
				let clone_boxed_data = BoxedData::alloc(self.capacity());
				clone_boxed_data.get_data_ptr().copy_from_nonoverlapping(boxed_data.get_data_ptr(), self.length as usize);
				Self {
					capacity: self.capacity,
					length: self.length,
					data: clone_boxed_data.into_inner().as_ptr()
				}
			}
		} else {
			Self {
				capacity: self.capacity,
				length: self.length,
				data: self.data
			}
		}
	}
}

impl Drop for ArcStringBuilder {
	fn drop(&mut self) {
		if let Some(boxed_data) = self.get_boxed_data() {
			unsafe {
				boxed_data.dealloc(self.capacity as usize);
			}
		}
	}
}

impl PartialEq for ArcStringBuilder {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialEq<str> for ArcStringBuilder {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<ArcString> for ArcStringBuilder {
	fn eq(&self, other: &ArcString) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for ArcStringBuilder {}

impl Hash for ArcStringBuilder {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl Debug for ArcStringBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.as_str())
	}
}

impl Display for ArcStringBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Write for ArcStringBuilder {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.push_str(s);
		Ok(())
	}
}

impl From<ArcString> for ArcStringBuilder {
	fn from(value: ArcString) -> Self {
		match value.try_take_boxed_data() {
			Ok(x) => {
				let len = unsafe {x.len()};
				ArcStringBuilder {
					capacity: len,
					length: len,
					data: x.into_inner().as_ptr()
				}
			}
			Err(value) => ArcStringBuilder::from(value.as_str())
		}
	}
}

impl From<char> for ArcStringBuilder {
	fn from(value: char) -> Self {
		value.encode_utf8(&mut [0; 4]).into()
	}
}

impl From<&str> for ArcStringBuilder {
	fn from(s: &str) -> Self {
		if let Some(sso) = ArcStringBuilder::try_new_sso(s) {
			sso
		} else {
			let len = s.len();
			let boxed_data = BoxedData::alloc(len);
			unsafe {
				boxed_data.get_data_ptr().as_ptr().copy_from_nonoverlapping(s.as_ptr(), len);
				Self {
					capacity: len as ulen,
					length: len as ulen,
					data: boxed_data.into_inner().as_ptr()
				}
			}
		}
	}
}

impl From<&mut str> for ArcStringBuilder {
	fn from(value: &mut str) -> Self {
		Self::from(value as &str)
	}
}

impl From<String> for ArcStringBuilder {
	fn from(value: String) -> Self {
		Self::from(value.as_str())
	}
}

impl Add for ArcStringBuilder {
	type Output = Self;
	fn add(mut self, rhs: Self) -> Self::Output {
		if self.is_empty() {
			rhs
		} else {
			self.push_str(rhs.as_str());
			self
		}
	}
}

impl Add<&str> for ArcStringBuilder {
	type Output = Self;
	fn add(mut self, rhs: &str) -> Self::Output {
		self.push_str(rhs);
		self
	}
}

impl Add<char> for ArcStringBuilder {
	type Output = Self;
	fn add(mut self, rhs: char) -> Self::Output {
		self.push_str(rhs.encode_utf8(&mut [0; 4]));
		self
	}
}

impl Borrow<str> for ArcStringBuilder {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl BorrowMut<str> for ArcStringBuilder {
	fn borrow_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl AsRef<str> for ArcStringBuilder {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}
