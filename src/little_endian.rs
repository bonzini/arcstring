use std::{num::NonZeroUsize, ptr::NonNull};

use crate::{MAX_SSO_LEN, boxed_data::Header};

/*
	the tag sits in the highest byte, which is the last of the string and thus
	cannot be a continuation byte

	if the highest three bits are 110, boxed
	if the highest three bits are 111, SSO(len < max)
	else, SSO(len == max)

	an SSO string is padded with 0xFF bytes, which never appear in UTF-8; as the
	padding sits in the highest bytes, the length is MAX_SSO_LEN minus the number
	of leading one bytes. counting bits and dividing by eight is enough, because
	the last byte of the string is never 0xFF and so contributes at most five
	leading ones
*/

const USIZE_BITS: usize = usize::BITS as usize;
const HIGH_BITS: usize = 0b111 << (USIZE_BITS - 3);
const BOXED_TAG: usize = 0b110 << (USIZE_BITS - 3);
pub const EMPTY: NonZeroUsize = NonZeroUsize::new(usize::MAX).unwrap();

#[inline(always)]
pub const fn try_encode_sso(s: &str) -> Option<NonZeroUsize> {
	let s = s.as_bytes();
	if s.len() > MAX_SSO_LEN {
		return None;
	}
	let mut data = [!0; MAX_SSO_LEN];
	unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), data.as_mut_ptr(), s.len()) };

	// the padding is empty for a string of MAX_SSO_LEN bytes, which is the only
	// case in which the encoding can be zero: MAX_SSO_LEN NUL bytes are boxed
	NonZeroUsize::new(usize::from_ne_bytes(data))
}

#[inline(always)]
pub fn encode_ptr(ptr: usize) -> NonZeroUsize {
	unsafe {NonZeroUsize::new_unchecked(ptr >> 3 | BOXED_TAG)}
}

#[inline(always)]
pub fn decode(val: &NonZeroUsize) -> &str {
	match val.get() & HIGH_BITS {
		BOXED_TAG => unsafe {
			let ptr = (val.get() << 3) as *const Header;
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				&(*ptr).data as *const _ as *const u8, (*ptr).len as usize
			))
		}
		_ => unsafe {
			let len = MAX_SSO_LEN - val.get().leading_ones() as usize / 8;
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				val as *const _ as *const u8, len
			))
		}
	}
}

pub fn as_ptr(val: usize) -> Option<NonNull<()>> {
	if val & HIGH_BITS == BOXED_TAG {
		Some(unsafe {NonNull::new_unchecked((val << 3) as *mut ())})
	} else {
		None
	}
}
