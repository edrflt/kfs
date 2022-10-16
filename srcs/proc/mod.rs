use crate::memory::allocator::Box;
use crate::vec::Vec;
use crate::VirtAddr;
use crate::wrappers::{_cli, _rst};

pub mod task;
pub mod process;
pub mod signal;

use process::{Process, zombify_running_process};
use task::{Task, TASKLIST, STACK_TASK_SWITCH, switch_task};

pub type Id = i32;

#[no_mangle]
pub unsafe extern "C" fn exit_fn() -> ! {
	_cli();
	TASKLIST.pop();
	let res = &TASKLIST.peek();
	if res.is_none() {
		todo!();
	}
	_rst();
	switch_task(&res.as_ref().unwrap().regs);
	/* Never goes there */
	loop {}
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn wrapper_fn() {
	core::arch::asm!("
	mov eax, [esp]
	add esp, 4
	call eax
	cli
	mov esp, STACK_TASK_SWITCH
	sub esp, 256
	jmp exit_fn",
	options(noreturn));
	/* Never goes there */
}

pub unsafe extern "C" fn exec_fn(func: VirtAddr, args_size: &Vec<usize>, mut args: ...) {
	let mut new_task: Task = Task::new();
	let res = TASKLIST.peek();
	if res.is_none() {
		todo!();
	}
	let regs = res.unwrap().regs;
	new_task.init(regs.eflags, regs.cr3);
	/* init_fn_task - Can't move to another function ??*/
	let sum: usize = args_size.iter().sum();
	new_task.regs.esp -= sum as u32;
	let mut nb = 0;
	for arg in args_size.iter() {
		let mut n: usize = *arg;
		while n > 0 {
			if arg / 4 > 0 {
				core::arch::asm!("mov [{esp} + {nb}], eax",
					esp = in(reg) new_task.regs.esp,
					nb = in(reg) nb,
					in("eax") args.arg::<u32>());
				n -= 4;
				nb += 4;
			}
			else if arg / 2 > 0 {
				core::arch::asm!("mov [{esp} + {nb}], ax",
					esp = in(reg) new_task.regs.esp,
					nb = in(reg) nb,
					in("ax") args.arg::<u16>());
				n -= 2;
				nb += 2;
			} else {
				todo!();
			}
		}
	}
	/* call function to wrapper_fn */
	new_task.regs.esp -= 4;
	core::arch::asm!("mov [{esp}], {func}",
		esp = in(reg) new_task.regs.esp,
		func = in(reg) func);
	TASKLIST.push(new_task);
}

#[macro_export]
macro_rules! size_of_args {
	($vector:expr, $name:expr) => { $vector.push(core::mem::size_of_val(&$name)); };
	($vector: expr, $x:expr, $($rest:expr),+) => {
		crate::size_of_args!($vector, $x); crate::size_of_args!($vector, $($rest),+)
	}
}

#[macro_export]
macro_rules! exec_fn {
	($func:expr, $($rest:expr),+) => {
		crate::wrappers::_cli();
		let mut args_size: crate::vec::Vec<usize> = crate::vec::Vec::new();
		crate::size_of_args!(args_size, $($rest),+);
		crate::proc::exec_fn($func, &args_size, $($rest),+);
		crate::wrappers::_sti()
	}
}
