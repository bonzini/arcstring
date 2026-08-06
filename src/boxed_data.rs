use core::{alloc::Layout, sync::atomic::{AtomicU32, fence}};
use std::{ptr::NonNull, sync::atomic::Ordering};

use crate::ulen;

#[repr(align(8))]
#[repr(C)]
pub struct Header {
	pub rc: AtomicU32,
	pub len: ulen,
	pub data: ()
}

/// A [`Header`] for a string that lives in static memory, so that a literal is
/// laid out exactly like a boxed string and decodes the same way.
#[repr(align(8))]
#[repr(C)]
pub struct StaticHeader<const N: usize> {
	/* the reference count is never touched, because a literal is not reference
	   counted; it is a plain u32 rather than an AtomicU32 so that the descriptor
	   has no interior mutability and can be promoted to static memory */
	rc: u32,
	len: ulen,
	data: [u8; N]
}

const _: () = assert!(size_of::<Header>() == core::mem::offset_of!(StaticHeader<0>, data));

impl<const N: usize> StaticHeader<N> {
	pub const fn new(s: &'static str) -> Self {
		assert!(s.len() == N, "the length of the string must be the size of the descriptor");
		assert!(N <= ulen::MAX as usize, "the string is too long for the length type being used");
		let mut result = Self { rc: 0, len: N as ulen, data: [0; N] };
		unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), result.data.as_mut_ptr(), N); }
		result
	}
}

fn layout_for_len(len: usize) -> Layout {
	Layout::new::<Header>().extend(Layout::from_size_align(len, 1).unwrap()).unwrap().0
}

pub(crate) struct BoxedData(NonNull<Header>);

impl BoxedData {
	pub fn from_ptr(ptr: NonNull<()>) -> Self {
		Self(ptr.cast())
	}

	pub fn alloc(capacity: usize) -> Self {
		unsafe {
			let header_ptr = NonNull::new(std::alloc::alloc(layout_for_len(capacity))).unwrap();
			Self(header_ptr.cast())
		}
	}

	#[must_use]
	pub unsafe fn realloc(self, old_capacity: usize, new_capacity: usize) -> Self {
		unsafe {
			let header_ptr = NonNull::new(std::alloc::realloc(self.0.as_ptr().cast(), layout_for_len(old_capacity), layout_for_len(new_capacity).size())).unwrap();
			Self(header_ptr.cast())
		}
	}

	pub unsafe fn dealloc(self, capacity: usize) {
		unsafe {
			std::alloc::dealloc(self.0.as_ptr().cast(), layout_for_len(capacity));
		}
	}

	pub unsafe fn finalize(self, len: ulen) -> usize {
		unsafe {
			self.0.write(Header {
				rc: AtomicU32::new(1),
				len,
				data: ()
			});
		}
		self.0.as_ptr() as usize
	}

	pub fn as_usize(self) -> usize {
		self.0.as_ptr() as usize
	}

	pub fn get_data_ptr(&self) -> NonNull<u8> {
		unsafe {self.0.byte_add(size_of::<Header>()).cast::<u8>()}
	}

	pub unsafe fn len(&self) -> ulen {
		unsafe {self.0.as_ref().len}
	}

	pub unsafe fn is_only_ref(&self) -> bool {
		unsafe {
			self.0.as_ref().rc.load(Ordering::Acquire) == 1
		}
	}

	pub unsafe fn increment_ref(self) {
		unsafe {
			if self.0.as_ref().rc.fetch_add(1, Ordering::Relaxed) >= u32::MAX / 2 {
				panic!("too many references to a single string");
			}
		}
	}

	pub unsafe fn destroy_ref(self) {
		unsafe {
			if self.0.as_ref().rc.fetch_sub(1, Ordering::Release) == 1 {
				// the last decrement needs to synchronize with the previous ones
				fence(Ordering::Acquire);
				std::alloc::dealloc(self.0.as_ptr().cast(), layout_for_len(self.len() as usize));
			}
		}
	}
}
