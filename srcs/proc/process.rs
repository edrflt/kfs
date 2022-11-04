use core::fmt;

use crate::KHEAP;
use crate::vec::Vec;
use crate::memory::{MemoryZone, Stack};
use crate::memory::paging::{PAGE_WRITABLE, free_pages};
use crate::memory::allocator::Box;

use crate::proc::task::TASKLIST;

use crate::proc::Id;
use crate::proc::signal::{Signal, SignalType, SignalHandler};

use crate::errno::ErrNo;

pub static mut NEXT_PID: Id = 0;
pub static mut MASTER_PROCESS: Process = Process::new();

pub type Pid = Id;

#[derive(Debug)]
pub enum Status {
	Disable,
	Run,
	Zombie,
	Thread
}

pub struct Process {
	pub pid: Pid,
	pub state: Status,
	pub parent: *mut Process,
	pub childs: Vec<Box<Process>>,
	pub stack: MemoryZone,
	pub heap: MemoryZone,
	pub signals: Vec<Signal>,
	pub signal_handlers: Vec<SignalHandler>,
	pub owner: Id
}

impl Process {
	pub const fn new() -> Self {
		Self {
			pid: 0,
			state: Status::Disable,
			parent: core::ptr::null_mut(),
			childs: Vec::new(),
			stack: MemoryZone::new(),
			heap: MemoryZone::new(),
			signals: Vec::new(),
			signal_handlers: Vec::new(),
			owner: 0
		}
	}

	pub fn get_nb_process(&self) -> usize {
		let mut ret: usize = 1;
		for process in self.childs.iter() {
			ret += process.get_nb_process()
		}
		ret
	}

	pub fn print_tree(&self) {
		crate::kprintln!("{}", self);
		for process in self.childs.iter() {
			process.print_tree();
		}
	}

	pub fn search_from_pid(&mut self, pid: Id) -> Result<&mut Process, ErrNo> {
		if self.pid == pid {
			return Ok(self);
		}
		for process in self.childs.iter_mut() {
			let res = process.search_from_pid(pid);
			if res.is_ok() {
				return res;
			}
		}
		Err(ErrNo::ESRCH)
	}

	/* TODO: next_pid need to check overflow and if other pid is available */
	pub unsafe fn init(&mut self, parent: *mut Process, owner: Id) {
		self.pid = NEXT_PID;
		self.state = Status::Run;
		self.parent = parent;
		self.stack = <MemoryZone as Stack>::init(0x1000, PAGE_WRITABLE, false);
		self.heap = KHEAP;
//		self.heap = <MemoryZone as Heap>::init(0x1000, PAGE_WRITABLE, false, &mut KALLOCATOR);
		self.owner = owner;
		NEXT_PID += 1;
	}

	pub unsafe fn zombify(&mut self, wstatus: i32) {
		if self.parent.is_null() {
			todo!();
		}
		let parent: &mut Process = &mut *self.parent;
		while self.childs.len() > 0 {
			/* TODO: DON'T MOVE THREADS AND REMOVE THEM */
			let res = self.childs.pop();
			if res.is_none() {
				todo!();
			}
			parent.childs.push(res.unwrap());
			let len = parent.childs.len();
			parent.childs[len - 1].parent = self.parent;
		}
		/* Don't remove and wait for the parent process to do wait4() -> Zombify */
		self.state = Status::Zombie;
		Signal::send_to_process(parent, self.pid, SignalType::SIGCHLD, wstatus);
		free_pages(self.stack.offset, self.stack.size / 0x1000);
	}

	pub unsafe fn remove(&mut self) {
		let parent: &mut Process = &mut *self.parent;
		let mut i = 0;
		while i < parent.childs.len() {
			let ptr: *mut Process = parent.childs[i].as_mut();
			if ptr == &mut *self {
				break ;
			}
			i += 1;
		}
		if i == parent.childs.len() {
			todo!(); // Problem
		}
		parent.childs.remove(i);
	}

	pub unsafe fn get_signal(&mut self, signal: SignalType) -> Result<Signal, ErrNo> {
		let mut i = 0;
		while i < self.signals.len() {
			if self.signals[i].sigtype == signal {
				return Ok(self.signals.remove(i));
			}
			i += 1;
		}
		Err(ErrNo::EAGAIN)
	}

	pub unsafe fn get_signal_from_pid(&mut self, pid: Id, signal: SignalType) -> Result<Signal, ErrNo> {
		MASTER_PROCESS.search_from_pid(pid)?; /* Return ErrNo::ESRCH if doesn't exist */
		let mut i = 0;
		while i < self.signals.len() {
			if self.signals[i].sender == pid && self.signals[i].sigtype == signal {
				return Ok(self.signals.remove(i));
			}
			i += 1;
		}
		Err(ErrNo::EAGAIN)
	}
}

impl fmt::Display for Process {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:10} - {:10} - {:?}", self.pid, self.owner, self.state)
	}
}

pub unsafe fn get_running_process() -> *mut Process {
	let res = TASKLIST.front_mut();
	if res.is_none() {
		todo!();
	}
	let task = res.unwrap();
	task.process
}

pub unsafe fn zombify_running_process(status: i32) {
	(*get_running_process()).zombify(status);
}

pub unsafe fn get_signal_running_process(pid: Id, signal: SignalType) -> Result<Signal, ErrNo> {
	let process = &mut *get_running_process();
	if pid == -1  {
		process.get_signal(signal)
	} else if pid > 0 {
		process.get_signal_from_pid(pid, signal)
	} else if pid == 0 {
		todo!();
	}
	else {
		todo!();
	}
}

pub unsafe fn print_all_process() {
	crate::kprintln!("       PID        OWNER   STATUS");
	MASTER_PROCESS.print_tree();
}

pub unsafe fn get_nb_process() -> usize {
	MASTER_PROCESS.get_nb_process()
}
