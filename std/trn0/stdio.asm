global jaguar_rest
jaguar_rest:
  mov rax, 60
  syscall
  ret

global println
; rdi --> string to print
println:
  mov rdi, rdi
  call print
  mov rax, 1
  mov rdi, 1
  mov rsi, nl
  mov rdx, 1
  syscall
  ret

global print
; rdi --> string to print
print:
  mov rdi, rdi
  call str_len
  mov rdx, rax
  mov rax, 1
  mov rsi, rdi
  mov rdi, 1
  syscall
  ret

global str_len
; rdi --> string to get length of
; rax --> return: length of string
str_len:
  xor rcx, rcx                              ; the loop counter
.loop:
  mov al, byte [rdi + rcx]
  test al, al                               ; check for null termination
  jz .done
  inc rcx
  jmp .loop

.done:
  mov rax, rcx
  ret

global write_int
; rdi --> integer to write
; rsi --> file descriptor
write_int:
  push rbx
  push rdi
  push rsi
  push rdx
  push rcx
  push r8
  push r9
  sub rsp, 32

  mov rcx, 10
  mov rax, rdi
  lea rdi, [rsp + 31]
  mov byte [rdi], 10
  dec rdi
.loop:
  xor rdx, rdx
  div rcx

  add dl, '0'
  mov [rdi], dl
  dec rdi
  test rax, rax
  jnz .loop

  inc rdi
  mov rsi, rdi
  mov rdx, 32
  add rdx, rsp
  sub rdx, rdi
  xor rcx, rcx
.loop2:
  mov al, byte [rsi + rcx]
  cmp al, 10
  je .donel
  mov [rdi + rcx], al
  inc rcx
  jmp .loop2
  
.donel:
  call print
  add rsp, 32

  pop r9
  pop r8
  pop rcx
  pop rdx
  pop rsi
  pop rdi
  pop rbx
  ret

global input
; rdi --> number of bytes to read
; rsi --> prompt
input:
  cmp rdi, 1024
  jg .err
  
  push rdi
  mov rdi, rsi
  call print
  pop rdx
  mov rax, 0
  mov rdi, 0
  mov rsi, temp
  syscall
  xor rcx, rcx
.loop:
  mov al, byte [temp + rcx]
  cmp al, 10
  je .done
  mov [input_ret + rcx], al
  inc rcx
  jmp .loop
  
.done:
  lea rax, input_ret
  ret
.err:
  push rdi
  lea rdi, max_err_msg
  call print
  pop rdi
  call write_int
  
  mov rax, 60
  mov rdi, 10
  syscall

global mem_get
; rdi ---> size of bytes to allocate
; rax ---> returns: pointer to allocated space
mem_get:
  mov rcx, rdi                              ; save the size

  mov rax, 9
  xor rdi, rdi
  mov rsi, rdi
  mov rsi, rcx
  mov rdx, 3
  mov r10, 0x22
  xor r8, r8
  xor r9, r9
  syscall
  cmp rax, -1
  je .err
  ret
.err:
  lea rdi, alloc_failed
  call println
  mov rax, 60
  mov rdi, 10
  syscall
section .note.GNU-stack
section .bss
temp : resb 1024
input_ret: resb 1024
section .rodata
nl: db "", 10, 0
max_err_msg: db "Error: Bytes to read is out of bounds: ", 0
alloc_failed: db "Error: Allocation failed", 0
