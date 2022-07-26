#![feature(const_mut_refs)]
#![feature(specialization)]
#![feature(box_syntax)]
#![feature(ptr_internals)]
#![feature(fundamental)]
#![feature(lang_items)]
#![no_std]
#![allow(dead_code)]
#![no_main]

/*  Custom test framwork    */
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
mod test;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
	kprintln!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}
	io::outb(0xf4, 0x10);
}

#[cfg(test)]
pub trait Testable {
	fn run(&self) -> ();
}

#[cfg(test)]
impl<T> Testable for T
where T: Fn(),
{
	fn run(&self) {
		kprint!("{}... ", core::any::type_name::<T>());
		self();
		change_color!(Color::Green, Color::Black);
		kprintln!("[ok]");
		change_color!(Color::White, Color::Black);
	}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}


/*  Modules import  */
mod io;
mod keyboard;
mod vga_buffer;
mod gdt;
mod cli;
mod paging;
mod interrupts;
mod kmemory;
mod multiboot;
mod allocator;

/*  Modules used function and variable  */
use paging::{init_paging, alloc_pages_at_addr, page_directory, VirtAddr};
use allocator::{linked_list::LinkedListAllocator, /*init_heap,*/ init_kheap};
use vga_buffer::color::Color;
use cli::Command;
use allocator::{Box};

#[global_allocator]
static mut ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

static mut KALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

/*  Code from boot section  */
#[allow(dead_code)]
extern "C" {
	static gdt_desc: u16;
	fn stack_bottom();
	fn stack_top();
	fn heap();
}

pub fn init_stack(stack_top: VirtAddr, stack_size: usize) -> VirtAddr {
	let mut nb_page: usize = stack_size / 4096;
	let stack_bottom: VirtAddr = stack_top - (stack_size - 1) as u32;
	if stack_size % 4096 != 0 {
		nb_page += 1;
	}
	alloc_pages_at_addr(stack_bottom, nb_page).expect("unable to allocate pages for stack");
	stack_top
}

/*  Kernel initialisation   */
#[no_mangle]
pub extern "C" fn kinit() {
	kprintln!("kinit_start");
	kprintln!("multiboot:");
	multiboot::read_tags();
	kprintln!("init_paging");
	init_paging();
	kprintln!("init_heap");
	unsafe {init_kheap(heap as u32, 100 * 4096 , &mut ALLOCATOR)};
	unsafe {init_kheap(paging::alloc_pages(5).expect("Failed"), 100 * 4096 , &mut KALLOCATOR)};
	kprintln!("init_stack");
//	init_stack(0xffffffff, 8192); -> do not do that here now, page_table are there
//	unsafe{core::arch::asm!("mov esp, eax", in("eax") 0xffffffff as u32)};

	#[cfg(test)]
	test_main();

	#[cfg(not(test))]
	kmain();
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {

	kprintln!("Hello World of {}!", 42);

	change_color!(Color::Red, Color::White);
	kprintln!("Press Ctrl-{} to navigate to the second workspace", '2');
	change_color!(Color::White, Color::Black);

	#[cfg(not(test))]
	test();

	let x = Box::new(5 as u64);
	kprintln!("New value: {}", x);
	kprint!("$> ");
	loop {
		if keyboard::keyboard_event() {
			let charcode = keyboard::handle_event();
			clihandle!(charcode);
		}
	}
}

/*  Function to put all tests and keep main clean */
#[cfg(not(test))]
fn test() {
	let x = Box::new(10);
	kprintln!("Virtual box: {}", x);
	let y = Box::knew(5);//, crate::allocator::KGlobal);
	kprintln!("Physical box: {}", y);
//	unsafe {
//	}
}

