global jump_usermode
global userfunc
global userfunc_end

jump_usermode:
	mov ax, (5 * 8) | 3 ; ring 3 data with bottom 2 bits set for ring 3
	mov ds, ax
	mov es, ax 
	mov fs, ax 
	mov gs, ax ; SS is handled by iret
 
	; set up the stack frame iret expects
	mov eax, esp
	push (5 * 8) | 3 ; data selector
	push eax ; current esp
	pushf ; eflags
	push (4 * 8) | 3 ; code selector (ring 3 code with bottom 2 bits set for ring 3)
	push 0x400000 ; instruction address to return to
	iret

userfunc:
	mov eax, 5
	int 0x80
userfunc_end:	
