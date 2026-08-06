use core::{alloc::Layout, marker::PhantomData, ops::{Deref, DerefMut}, num::NonZeroUsize, sync::atomic::{AtomicU32, fence}};
use std::{ptr::NonNull, sync::atomic::Ordering};

use crate::ulen;

/* the string is a slice at the end of the header, so that a reference to the
   header covers the string as well: a reference to a header without it would
   not be allowed to reach the data, however the pointer is derived from it.

   the pointer stored in an ArcString has to stay thin, though, so it points to
   the sized Header<[u8; 0]> and the length of the slice is supplied again every
   time the header is borrowed as a whole */
#[repr(align(8))]
#[repr(C)]
pub struct Header<T: ?Sized = [u8; 0]> {
	pub rc: AtomicU32,
	pub len: ulen,
	pub data: T
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

/* NonNull::map_addr() works on NonZeroUsize, which the tagging and untagging of
   an address never has to care about: f must never return zero */
#[inline(always)]
pub fn map_addr(ptr: NonNull<Header>, f: fn(usize) -> usize) -> NonNull<Header> {
	ptr.map_addr(|addr| {
		let addr = f(addr.get());
		debug_assert!(addr != 0);
		unsafe {NonZeroUsize::new_unchecked(addr)}
	})
}

/* the layout is padded to the alignment, because that is the size of the
   Header<[u8]> that borrows the allocation */
fn layout_for_len(len: usize) -> Layout {
	Layout::new::<Header>().extend(Layout::from_size_align(len, 1).unwrap()).unwrap().0.pad_to_align()
}

/* the pointer is raw rather than a &mut, because a reference passed to a
   function is strongly protected for the duration of the call, and protected
   memory must not be deallocated: the references below are minted on demand and
   never held across a reallocation */
pub(crate) struct BoxedData<'a>(NonNull<Header<[u8]>>, PhantomData<&'a mut Header<[u8]>>);

/* the header is only ever borrowed for as long as it takes to touch a field, so
   that no reference is live across a reallocation or a deallocation */
impl Deref for BoxedData<'_> {
	type Target = Header<[u8]>;
	fn deref(&self) -> &Header<[u8]> {
		unsafe {self.0.as_ref()}
	}
}

impl DerefMut for BoxedData<'_> {
	fn deref_mut(&mut self) -> &mut Header<[u8]> {
		unsafe {self.0.as_mut()}
	}
}

impl BoxedData<'_> {
	/* the capacity is the length the slice is borrowed with, and has to be the
	   one the allocation was made with */
	pub unsafe fn from_ptr_with_capacity(ptr: NonNull<Header>, capacity: usize) -> Self {
		let ptr = std::ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast::<u8>(), capacity) as *mut Header<[u8]>;
		Self(unsafe {NonNull::new_unchecked(ptr)}, PhantomData)
	}

	/// # Safety
	///
	/// The header must have been finalized, so that the length it stores is the
	/// capacity it was allocated with.
	pub unsafe fn from_ptr(ptr: NonNull<Header>) -> Self {
		let len = unsafe {ptr.as_ref()}.len as usize;
		unsafe {Self::from_ptr_with_capacity(ptr, len)}
	}

	#[must_use]
	pub fn alloc(capacity: usize) -> Self {
		assert!(ulen::try_from(capacity).is_ok(), "the string is too long for the length type being used");
		let ptr = NonNull::new(unsafe {std::alloc::alloc(layout_for_len(capacity))}).unwrap();
		unsafe {Self::from_ptr_with_capacity(ptr.cast(), capacity)}
	}

	#[must_use]
	pub fn realloc(self, new_capacity: usize) -> Self {
		assert!(ulen::try_from(new_capacity).is_ok(), "the string is too long for the length type being used");
		let old_layout = Layout::for_value(&*self);
		let new_layout = layout_for_len(new_capacity);
		let ptr = if new_layout.size() == old_layout.size() {
			/* the padding absorbs the difference: only the capacity that the slice is
			   borrowed with has to change */
			self.into_inner()
		} else {
			let ptr = unsafe {std::alloc::realloc(self.0.cast::<u8>().as_ptr(), old_layout, new_layout.size())};
			NonNull::new(ptr).unwrap().cast()
		};
		unsafe {Self::from_ptr_with_capacity(ptr.cast(), new_capacity)}
	}

	pub fn dealloc(self) {
		let layout = Layout::for_value(&*self);
		unsafe {std::alloc::dealloc(self.0.cast::<u8>().as_ptr(), layout)};
	}

	#[must_use]
	pub fn finalize(mut self) -> NonNull<Header> {
		/* from now on the header is borrowed from its own length, so the two have
		   to agree */
		let len = self.data.len();
		self.rc = AtomicU32::new(1);
		self.len = len as ulen;
		self.into_inner()
	}

	#[must_use]
	pub fn into_inner(self) -> NonNull<Header> {
		self.0.cast()
	}

	/* reading and writing are kept apart: a &mut to the data would invalidate
	   every &str handed out so far, and as_str() only ever needs to read */
	pub fn get_data_ptr(&self) -> *const u8 {
		(&raw const self.data).cast()
	}

	pub fn get_data_ptr_mut(&mut self) -> *mut u8 {
		(&raw mut self.data).cast()
	}

	pub fn len(&self) -> ulen {
		self.len
	}

	pub fn capacity(&self) -> usize {
		/* the allocation was padded to the alignment of the header, so it holds a few
		   bytes more than were asked for */
		Layout::for_value(&**self).size() - size_of::<Header>()
	}

	pub fn is_only_ref(&self) -> bool {
		self.rc.load(Ordering::Acquire) == 1
	}

	pub fn increment_ref(&mut self) {
		if self.rc.fetch_add(1, Ordering::Relaxed) >= u32::MAX / 2 {
			panic!("too many references to a single string");
		}
	}

	pub fn destroy_ref(self) {
		if self.rc.fetch_sub(1, Ordering::Release) == 1 {
			// the last decrement needs to synchronize with the previous ones
			fence(Ordering::Acquire);
			self.dealloc();
		}
	}
}
