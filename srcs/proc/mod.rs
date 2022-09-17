use crate::memory::MemoryZone;
use crate::memory::paging::{alloc_page, PAGE_WRITABLE};
use crate::vec::Vec;
use crate::VirtAddr;

enum Status {
	Run,
	Zombie,
	Thead
}

type Id = u32;

struct Signal {
}

struct Process {
	pid: Id,
	status: Status,
	parent: *const Process,
	childs: Vec<*mut Process>,
	stack: MemoryZone,
	heap: MemoryZone,
	signals: Vec<Signal>, /* TODO: VecDeque ? */
	owner: Id
}

static mut RUNNING_TASK: *const Task = core::ptr::null();
static mut MAIN_TASK: Task = Task::new();

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Registers {
	eax:	u32,
	ebx:	u32,
	ecx:	u32,
	edx:	u32,
	esi:	u32,
	edi:	u32,
	esp:	u32,
	ebp:	u32,
	eip:	u32,
	eflags:	u32,
	cr3:	u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Task {
	regs: Registers,
	next: *const Task
}

impl Task {
	const fn new() -> Self {
		Self {
			regs: Registers {
				eax: 0,
				ebx: 0,
				ecx: 0,
				edx: 0,
				esi: 0,
				edi: 0,
				esp: 0,
				ebp: 0,
				eip: 0,
				eflags: 0,
				cr3: 0
			},
			next: core::ptr::null()
		}
	}

	fn init(&mut self, addr: u32, flags: u32, page_dir: u32) {
		self.regs.eip = addr;
		self.regs.eflags = flags;
		self.regs.cr3 = page_dir;
		let res = alloc_page(PAGE_WRITABLE);
		if res.is_ok() {
			self.regs.esp = res.unwrap() + 0x1000;
		} else {
			todo!();
		}
		crate::kprintln!("init task: {:#x?}", self.regs);
	}
}

use crate::memory::allocator::Box;

pub unsafe fn init_tasking() {
	core::arch::asm!("mov {}, cr3", out(reg) MAIN_TASK.regs.cr3);
	core::arch::asm!("pushf",
					"mov {}, [esp]",
					"popf", out(reg) MAIN_TASK.regs.eflags);
	let mut other_task: Task = Task::new();
	other_task.init(dumb_main as u32, MAIN_TASK.regs.eflags, MAIN_TASK.regs.cr3);

	other_task.next = &MAIN_TASK;
	crate::kprintln!("other_task.next: {:#x?}", other_task.next);
	let alloc = Box::new(other_task);
	MAIN_TASK.next = &*alloc;
	crate::kprintln!("main_task.next: {:#x?}", MAIN_TASK.next);
	crate::kprintln!("other_task.next: {:#x?}", (*MAIN_TASK.next).next);
	crate::kprintln!("main_task.next: {:#x?}", (*(*MAIN_TASK.next).next).next);
	crate::kprintln!("other_task.next: {:#x?}", (*(*(*MAIN_TASK.next).next).next).next);
	RUNNING_TASK = &MAIN_TASK;
}

fn dumb_main() {
	crate::kprintln!("other task !");
	unsafe {next_task()};
}

extern "C" {
	fn switch_task(reg_from: *const Registers, reg_to: *const Registers);
}

pub unsafe fn next_task() {
	let last: *const Task = RUNNING_TASK;
	crate::kprintln!("M41n_t4sk: {:#x?}", (*RUNNING_TASK));
	crate::kprintln!("M41n_t4sk.n3xt: {:#x?}", (*RUNNING_TASK).next);
	crate::kprintln!("0th3r_t4sk: {:#x?}", (*(*RUNNING_TASK).next));
	crate::kprintln!("0th3r_t4sk.n3xt: {:#x?}", (*(*RUNNING_TASK).next).next);
	crate::kprintln!("RUNNING_TASK: {:#x?}", (*RUNNING_TASK).regs);
	crate::kprintln!("NEXT_RUNNING_TASK_PTR: {:#x?}", (*RUNNING_TASK).next);
	RUNNING_TASK = (*RUNNING_TASK).next;
	crate::kprintln!("NEXT_RUNNING_TASK: {:#x?}", (*RUNNING_TASK).regs);
	crate::kprintln!("PREV_RUNNING_TASK_PTR: {:#x?}", (*RUNNING_TASK).next);
	core::arch::asm!("cli");
	crate::kprintln!("switching...");
	switch_task(&(*last).regs, &(*RUNNING_TASK).regs);
	crate::kprintln!("PREV_RUNNING_TASK: {:#x?}", (*(*RUNNING_TASK).next).regs);
	core::arch::asm!("sti");
	crate::kprintln!("bang");
}

pub fn		exec_fn(addr: VirtAddr, func: VirtAddr, size: u32) {
}
