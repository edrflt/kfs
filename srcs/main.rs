#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(fmt_internals)]
#![feature(specialization)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(rustc_attrs)]
#![feature(box_syntax)]
#![feature(ptr_internals)]
#![feature(fundamental)]
#![feature(lang_items)]
#![no_std]
#![allow(dead_code)]
#![allow(incomplete_features)]
#![no_main]

/*  Custom test framwork    */
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

const GLOBAL_ALIGN: usize = 8;

/* Allocation tracking */
pub struct Tracker {
	allocation:			usize,
	allocated_bytes:	usize,
	freed:				usize,
	freed_bytes:		usize
}

static mut TRACKER: Tracker = Tracker {
	allocation: 0,
	allocated_bytes: 0,
	freed: 0,
	freed_bytes: 0
};

static mut KTRACKER: Tracker = Tracker {
	allocation: 0,
	allocated_bytes: 0,
	freed: 0,
	freed_bytes: 0
};

pub fn memory_state() {
	unsafe {
		kprintln!("\nAllocation: {} for {} bytes", KTRACKER.allocation, KTRACKER.allocated_bytes);
		kprintln!("Free:       {} for {} bytes", KTRACKER.freed, KTRACKER.freed_bytes);
	}
}

/*  Modules import  */
mod cli;
mod gdt;
mod keyboard;
mod memory;
mod multiboot;
mod vec;
mod string;
mod interrupts;
mod syscalls;
mod io;
mod vga_buffer;
mod pic;

#[cfg(test)]
mod test;

/*  Modules used function and variable  */
use memory::paging::{init_paging, page_directory};
use memory::allocator::linked_list::LinkedListAllocator;
use vga_buffer::color::Color;
use cli::Command;
use pic::setup_pic8259;

#[global_allocator]
static mut ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();
static mut KALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

/*  Code from boot section  */
#[allow(dead_code)]
extern "C" {
	fn stack_bottom();
	fn stack_top();
	fn heap();
	fn jump_usermode();
}

use crate::memory::{init_heap, init_stack, VirtAddr};
use crate::memory::paging::PAGE_WRITABLE;

use crate::interrupts::init_idt;

use crate::gdt::{KERNEL_BASE, gdt_desc, update_gdtr};
	
use crate::memory::paging::{alloc_pages_at_addr, PAGE_USER};

/*  Kernel initialisation   */
#[no_mangle]
pub extern "C" fn kinit() {
//	multiboot::read_tags();
	/* Init paging and remove identity paging */
	init_paging();
	/* Update gdtr with higher half kernel gdt addr */
	unsafe {
		update_gdtr();
		reload_gdt!();
		init_idt();
	}

	/* HEAP KERNEL */
	unsafe {init_heap(heap as u32, 100 * 4096, PAGE_WRITABLE, true, &mut KALLOCATOR)};
	let kstack_addr: VirtAddr = 0xffbfffff; /* stack kernel */
	let tmp = init_stack(kstack_addr, 8192, PAGE_WRITABLE, false);

	/* Reserve some spaces to push things before main */
	unsafe{core::arch::asm!("mov esp, eax", in("eax") kstack_addr - 256)};

	setup_pic8259();
	
	gdt::tss::init_tss(kstack_addr);
	reload_tss!();
/*	Kernel stack entry on tss */

//	kprintln!("tss size: {}", core::mem::size_of::<gdt::tss::Tss>());
	let userpage = alloc_pages_at_addr(0x400000, 1, PAGE_WRITABLE | PAGE_USER).expect("");
	unsafe {
		for i in 0..4095 {
			*((userpage + i) as *mut u8) = 0x90;
		}
		*((userpage + 4095) as *mut u8) = 0xfa;
	}
	memory::paging::print_pdentry(1);
//	unsafe {
//		core::arch::asm!("mov eax, 0x400000", "jmp eax");
//	}
	#[cfg(test)]
	test_main();

	#[cfg(not(test))]
	kmain();
}

#[no_mangle]
pub fn userfunc() {
	kprintln!("Usermode");
	unsafe{ core::arch::asm!("cli");}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
	unsafe {
		jump_usermode();
	}
	kprintln!("Hello World of {}!", 42);

	change_color!(Color::Red, Color::White);
	let workspace_msg = string::String::from("Press Ctrl-2 to navigate to the second workspace");
	kprintln!("{}", workspace_msg);
	change_color!(Color::White, Color::Black);

	kprint!("$> ");
	loop {
		unsafe{core::arch::asm!("hlt")};
	}
}
