use crate::memory::MemoryZone;
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

static mut RUNNING_TASK: *mut Task = core::ptr::null_mut();

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Registers {
	pub eax:	u32,
	pub ebx:	u32,
	pub ecx:	u32,
	pub edx:	u32,
	pub esi:	u32,
	pub edi:	u32,
	pub esp:	u32,
	pub ebp:	u32,
	pub eip:	u32,
	pub eflags:	u32,
	pub cr3:	u32
}

pub struct Task {
	pub regs: Registers,
	pub next_ptr: *mut Task,
	pub next: Option<Box<Task>>
}

impl Task {
	pub const fn new() -> Self {
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
			next_ptr: core::ptr::null_mut(),
			next: None
		}
	}

	pub fn init(&mut self, addr: VirtAddr, func: u32, size: u32, flags: u32, page_dir: u32) {
		self.regs.eip = func;
		self.regs.eflags = flags;
		self.regs.cr3 = page_dir;
		self.regs.esp = addr + size;
	}
}

use crate::memory::allocator::Box;

pub unsafe fn init_tasking(main_task: &mut Task) {
	core::arch::asm!("mov {}, cr3", out(reg) main_task.regs.cr3);
	core::arch::asm!("pushf",
					"mov {}, [esp]",
					"popf", out(reg) main_task.regs.eflags);
	RUNNING_TASK = &mut *main_task;
}

pub unsafe fn append_task(mut new_task: Task) {
	/* TODO: mutex lock prevent switch_task .. */
	let mut task: &mut Task = &mut *RUNNING_TASK;
	if !task.next_ptr.is_null() {
		crate::kprintln!("lol");
		while !task.next.is_none() {
			task = &mut *task.next_ptr;
		}
		new_task.next_ptr = task.next_ptr;
	} else {
		new_task.next_ptr = &mut *task;
	}
	new_task.next = None;
	task.next = Some(Box::new(new_task));
	task.next_ptr = &mut *(task.next.as_mut().unwrap()).as_mut();
}

pub unsafe fn remove_task() {/* exit ? */
	/* TODO: mutex lock prevent switch_task .. */
	crate::kprintln!("remove_task()");
	let mut prev_task: &mut Task = &mut *RUNNING_TASK;
	while prev_task.next_ptr != &mut *RUNNING_TASK {
		prev_task = &mut *prev_task.next_ptr;
	}
	(*prev_task).next_ptr = (*RUNNING_TASK).next_ptr;
	if (*RUNNING_TASK).next.is_none() {
		(*prev_task).next = None;
	} else {
		(*prev_task).next = Some((*RUNNING_TASK).next.take().unwrap());
	}
	/* TODO: free stack ? */
	RUNNING_TASK = &mut *prev_task;
	crate::kprintln!("task removed finished");
	loop {} /* waiting for switch - TODO: replace by int ? */
}

extern "C" {
	fn switch_task(reg_from: *const Registers, reg_to: *const Registers);
}

#[no_mangle]
pub unsafe extern "C" fn next_task() {
	let last: *const Task = RUNNING_TASK;
	RUNNING_TASK = (*RUNNING_TASK).next_ptr;
//	crate::kprintln!("Running task: {:#x?}", RUNNING_TASK);
//	crate::kprintln!("Running task: {:#x?}", (*RUNNING_TASK).regs);
	core::arch::asm!("cli");
	crate::kprintln!("switching...");
	switch_task(&(*last).regs, &(*RUNNING_TASK).regs);
	core::arch::asm!("sti");
}

pub fn		exec_fn(addr: VirtAddr, func: VirtAddr, size: u32) {
	unsafe {
		let mut other_task: Task = Task::new();
		other_task.init(addr, func, size, (*RUNNING_TASK).regs.eflags, (*RUNNING_TASK).regs.cr3);
		append_task(other_task);
	}
}
