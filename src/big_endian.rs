use std::ptr::NonNull;

use crate::{MAX_SSO_LEN, boxed_data::{Header, map_addr}};

/*
	the tag lives in the first byte of the string, which is the highest byte;
	the first byte of a UTF-8 string is never a continuation byte, so 10xxxxxx
	is free to be used as a tag:
	if the highest three bits are 100, pointer to a heap allocated Header
	if the highest three bits are 101, pointer to a static Header
	else, SSO

	both kinds of pointer have the same layout, so only reference counting cares
	about the difference between the two tags

	an SSO string is padded with 0xFF bytes, which never appear in UTF-8; as the
	padding sits in the lowest bytes, the length is MAX_SSO_LEN minus the number
	of trailing one bytes. counting bits and dividing by eight is enough, because
	the last byte of the string is never 0xFF and so contributes at most five
	trailing ones
*/

const TAG_POS: u32 = usize::BITS - 3;
const TAG_BITS: usize = 0b111 << TAG_POS;
const BOXED_TAG: usize = 0b100 << TAG_POS;
const LITERAL_TAG: usize = 0b101 << TAG_POS;
const HEADER_BITS: usize = 0b110 << TAG_POS;
const HEADER_TAG: usize = 0b100 << TAG_POS;
pub const EMPTY: NonNull<Header> = encode_inline([!0; MAX_SSO_LEN]).unwrap();

#[inline(always)]
pub const fn encode_inline(data: [u8; MAX_SSO_LEN]) -> Option<NonNull<Header>> {
	// an SSO string is not a pointer: only the address bits of the NonNull are used,
	// and the provenance is never needed because it is never dereferenced
	NonNull::new(std::ptr::without_provenance_mut(usize::from_ne_bytes(data)))
}

#[inline(always)]
pub const fn try_encode_sso(s: &str) -> Option<NonNull<Header>> {
	let s = s.as_bytes();
	if s.len() > MAX_SSO_LEN {
		return None;
	}
	let mut data = [!0; MAX_SSO_LEN];
	unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), data.as_mut_ptr(), s.len()) };

	// the padding is empty for a string of MAX_SSO_LEN bytes, which is the only
	// case in which the encoding can be zero: MAX_SSO_LEN NUL bytes are boxed
	encode_inline(data)
}

/* rotating by TAG_POS moves the three always-zero low bits of the pointer onto
   the tag; map_addr() keeps the provenance of the allocation across the round trip */
#[inline(always)]
pub fn encode_ptr(ptr: NonNull<Header>) -> NonNull<Header> {
	map_addr(ptr, |addr| addr.rotate_left(TAG_POS) | BOXED_TAG)
}

#[inline(always)]
pub fn encode_literal(ptr: NonNull<Header>) -> NonNull<Header> {
	map_addr(ptr, |addr| addr.rotate_left(TAG_POS) | LITERAL_TAG)
}

#[inline(always)]
fn decode_ptr(val: NonNull<Header>) -> NonNull<Header> {
	map_addr(val, |addr| addr.rotate_right(TAG_POS) & !0b111)
}

#[inline(always)]
pub fn decode(val: &NonNull<Header>) -> &str {
	match val.addr().get() & HEADER_BITS {
		HEADER_TAG => unsafe {
			let ptr = decode_ptr(*val).as_ptr();
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				(&raw const (*ptr).data).cast::<u8>(), (*ptr).len as usize
			))
		}
		_ => unsafe {
			let len = MAX_SSO_LEN - val.addr().get().trailing_ones() as usize / 8;
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				val as *const _ as *const u8, len
			))
		}
	}
}

#[inline(always)]
pub fn as_ptr(val: NonNull<Header>) -> Option<NonNull<Header>> {
	if val.addr().get() & TAG_BITS == BOXED_TAG {
		Some(decode_ptr(val))
	} else {
		None
	}
}

#[inline(always)]
pub fn as_static(val: &NonNull<Header>) -> Option<&'static str> {
	if val.addr().get() & TAG_BITS == LITERAL_TAG {
		Some(unsafe {
			let ptr = decode_ptr(*val).as_ptr();
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				(&raw const (*ptr).data).cast::<u8>(), (*ptr).len as usize
			))
                })
	} else {
		None
	}
}
