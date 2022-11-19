use core::slice;

pub extern "C" fn sys_write(_file_descriptor: u32, buffer: *const u8, len: u32) -> usize
{
	unsafe {
		let len = len as usize;
		let buffer = slice::from_raw_parts(buffer, len);
		for i in 0..len
		{
			crate::kprint!("{}", buffer[i] as char);
		}
	}
	0
}
