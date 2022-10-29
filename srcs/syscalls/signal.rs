use crate::proc::process::{MASTER_PROCESS, Process, Pid, get_running_process};
use crate::proc::signal::{Signal, SignalType, get_signal_type};
use crate::wrappers::{_cli, _sti};

pub extern "C" fn sys_kill(pid: Pid, signal: i32) -> i32 {
	if pid > 0 { /* Send to a specific process */
		unsafe {
			let sender_pid = (*get_running_process()).pid;
			let res = get_signal_type(signal);
			if res.is_err() {
				return -(res.err().unwrap() as i32)
			}
			let signal_type = res.unwrap();
			if signal_type == SignalType::SIGKILL {
				let res = MASTER_PROCESS.search_from_pid(pid);
				if res.is_err() {
					return -(res.err().unwrap() as i32);
				}
				let process: &mut Process = res.unwrap();
				_cli();
				process.zombify(0); /* TODO: setup status */
				// TODO: Remove task etc..
				_sti();
				return pid;
			} else {
				let res = Signal::send_to_pid(pid, sender_pid, signal_type, 0);
				if res.is_err() {
					return -(res.err().unwrap() as i32);
				}
				return res.unwrap() as i32;
			}
		}
	} else if pid == 0 { /* Send to every process in process group */
		todo!();
	} else if pid == -1 { /* Send to every process that has permission */
		todo!();
	} else { /* pid > -1: Send to every process in process group where is -pid */
		todo!();
	}
}
