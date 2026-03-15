use std::{num::NonZeroUsize, ptr::NonNull};

use crate::{MAX_SSO_LEN, boxed_data::Header};

/*
	if the highest three bits are 110, boxed
	if the highest three bits are 111, SSO(len < max)
	else, SSO(len == max)
*/

const USIZE_BITS: usize = usize::BITS as usize;
const HIGH_BITS: usize = 0b111 << (USIZE_BITS - 3);
const SSO_TAG: usize = 0b111 << (USIZE_BITS - 3);
const BOXED_TAG: usize = 0b110 << (USIZE_BITS - 3);
const SSO_TAG_U8: u8 = 0b11100000;
pub const EMPTY: NonZeroUsize = NonZeroUsize::new(SSO_TAG).unwrap();

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub const fn try_encode_sso(s: &str) -> Option<NonZeroUsize> {
	NonZeroUsize::new(match s.len() {
		0 => usize::from_ne_bytes([0, 0, 0, SSO_TAG_U8]),
		1 => usize::from_ne_bytes([s.as_bytes()[0], 0, 0, SSO_TAG_U8 | 1]),
		2 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], 0, SSO_TAG_U8 | 2]),
		3 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], SSO_TAG_U8 | 3]),
		4 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3]]),
		_ => 0
	})
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub const fn try_encode_sso(s: &str) -> Option<NonZeroUsize> {
	NonZeroUsize::new(match s.len() {
		0 => usize::from_ne_bytes([0, 0, 0, 0, 0, 0, 0, SSO_TAG_U8]),
		1 => usize::from_ne_bytes([s.as_bytes()[0], 0, 0, 0, 0, 0, 0, SSO_TAG_U8 | 1]),
		2 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], 0, 0, 0, 0, 0, SSO_TAG_U8 | 2]),
		3 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], 0, 0, 0, 0, SSO_TAG_U8 | 3]),
		4 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3], 0, 0, 0, SSO_TAG_U8 | 4]),
		5 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3], s.as_bytes()[4], 0, 0, SSO_TAG_U8 | 5]),
		6 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3], s.as_bytes()[4], s.as_bytes()[5], 0, SSO_TAG_U8 | 6]),
		7 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3], s.as_bytes()[4], s.as_bytes()[5], s.as_bytes()[6], SSO_TAG_U8 | 7]),
		8 => usize::from_ne_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3], s.as_bytes()[4], s.as_bytes()[5], s.as_bytes()[6], s.as_bytes()[7]]),
		_ => 0
	})
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
		SSO_TAG => unsafe {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				val as *const _ as *const u8, val.get() << 4 >> (USIZE_BITS - 8 + 4)
			))
		}
		_ => unsafe {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				val as *const _ as *const u8, MAX_SSO_LEN
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
