use crate::io::{
outb
};

/*
** I/O port     Usage
** 0x40         Channel 0 data port (read/write)
** 0x41         Channel 1 data port (read/write)
** 0x42         Channel 2 data port (read/write)
** 0x43         Mode/Command register (write only, a read is ignored)
*/
const CHAN0_DATA:	u8 = 0x40;
const CHAN1_DATA:	u8 = 0x41;
const CHAN2_DATA:	u8 = 0x42;
const MODE_CMD:		u8 = 0x43;

/*
**	Bits         Usage
**	6 and 7      Select channel :
**	                0 0 = Channel 0
**	                0 1 = Channel 1
**	                1 0 = Channel 2
**	                1 1 = Read-back command (8254 only)
**	4 and 5      Access mode :
**	                0 0 = Latch count value command
**	                0 1 = Access mode: lobyte only
**	                1 0 = Access mode: hibyte only
**	                1 1 = Access mode: lobyte/hibyte
**	1 to 3       Operating mode :
**	                0 0 0 = Mode 0 (interrupt on terminal count)
**	                0 0 1 = Mode 1 (hardware re-triggerable one-shot)
**	                0 1 0 = Mode 2 (rate generator)
**	                0 1 1 = Mode 3 (square wave generator)
**	                1 0 0 = Mode 4 (software triggered strobe)
**	                1 0 1 = Mode 5 (hardware triggered strobe)
**	                1 1 0 = Mode 2 (rate generator, same as 010b)
**	                1 1 1 = Mode 3 (square wave generator, same as 011b)
**	0            BCD/Binary mode: 0 = 16-bit binary, 1 = four-digit BCD
*/

pub const CHANNEL_0:		u8 = 0b00	<< 6;
pub const CHANNEL_1:		u8 = 0b01	<< 6;
pub const CHANNEL_2:		u8 = 0b10	<< 6;
pub const READBACK:			u8 = 0b11	<< 6;
 
pub const ACC_LATCHCOUNT:	u8 = 0b00	<< 4;
pub const ACC_LOBONLY:		u8 = 0b01	<< 4;
pub const ACC_HIBONLY:		u8 = 0b10	<< 4;
pub const ACC_LOBHIB:		u8 = 0b11	<< 4;
 
pub const MODE_0:			u8 = 0b000	<< 1;
pub const MODE_1:			u8 = 0b001	<< 1;
pub const MODE_2:			u8 = 0b010	<< 1;
pub const MODE_3:			u8 = 0b011	<< 1;
pub const MODE_4:			u8 = 0b100	<< 1;
pub const MODE_5:			u8 = 0b101	<< 1;
pub const MODE_6:			u8 = MODE_2;
pub const MODE_7:			u8 = MODE_3;

pub fn set_pit(channel: u8, access: u8, mode: u8, data: u16) {
	let port: u16 = match channel {
		CHANNEL_0	=> CHAN0_DATA.into(),
		CHANNEL_1	=> CHAN1_DATA.into(),
		CHANNEL_2	=> CHAN2_DATA.into(),
		_			=>	panic!("PIT Channel {channel} doesn't exist"),
	};
/*	Set cmd mod selected */
	outb(MODE_CMD as u16, channel | access | mode);	

/*	Sending data to commands */
	outb(port, (data & 0xff) as u8);
	outb(port, ((data & 0xff00) >> 8) as u8);
}
