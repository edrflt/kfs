use core::ptr::{NonNull};
use core::alloc::{Layout, };
use crate::GLOBAL_ALIGN;
 use crate::allocator::{
Allocator,
AllocError,
Global
};

#[cfg(test)]
pub mod test;

pub struct Vec<T, A: Allocator = crate::allocator::Global> {
	ptr: NonNull<T>,
	capacity: usize,
	len: usize,
	alloc: A
}

pub fn test() {
	let x = Vec::<i32>::with_capacity(5);
}

impl<T> Vec<T> {

	pub fn new() -> Vec<T> {
		Vec {
			ptr: NonNull::<T>::dangling(),
			capacity: 0,
			len: 0,
			alloc: Global
		}
	}

	pub fn with_capacity(capacity: usize) -> Vec<T> {
		Vec {
			ptr: Self::with_capacity_in(capacity, &Global),
			capacity: capacity,
			len: 0,
			alloc: Global
		}
	}

	pub fn capacity(&self) -> usize {
		self.capacity
	}
	
	pub fn len(&self) -> usize {
		self.len
	}

	pub fn push(&mut self, value: T) {
		todo!()
	}

	pub fn pop(&mut self) -> Option<T> {
		todo!()
	}

	pub fn reserve(&mut self, additional: usize) {
		todo!()
	}

	pub fn insert(&mut self, index: usize, element: T) {
		todo!()
	}

	pub fn remove(&mut self, index: usize) -> T {
		todo!()
	}
}


impl<T, A: Allocator> Vec<T,A> {
	
	pub fn with_capacity_in(capacity: usize, alloc: &dyn Allocator) -> NonNull<T> {
		match Self::try_alloc(capacity, alloc) {
			Ok(x) => x,
			Err(_) => panic!("Allocation failed")
		}
	}

	fn try_alloc(capacity: usize, alloc: &dyn Allocator) -> Result<NonNull<T>, AllocError> {
		let layout = Layout::from_size_align(capacity, GLOBAL_ALIGN).unwrap();
		match alloc.allocate(layout) {
			Ok(res) => Ok(res.cast()),
			Err(_) => Err(AllocError{})
		}
	}
}
