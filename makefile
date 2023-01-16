DEBUG_OUT 	= target/amd64/debug/kernel
SRC_ROOT 	= $(abspath .)
BLOCK_SIZE 	= 4096

.PHONY: all 
all: qemu

.PHONY: debug_build
debug_build:
	@cargo build --target ${SRC_ROOT}/amd64.json
	@cp $(DEBUG_OUT) $(DEBUG_OUT).stripped
	@objcopy $(DEBUG_OUT).stripped -O binary 
	@truncate -s %$(BLOCK_SIZE) $(DEBUG_OUT).stripped

.PHONY: qemu
qemu: debug_build
	qemu-system-x86_64 -drive format=raw,file=$(DEBUG_OUT).stripped -s -S --no-reboot -m 4G -monitor stdio -d in_asm,int,unimp,mmu,cpu_reset,guest_errors,page -D qemu.log

gdb: debug_build
	x86_64-unknown-elf-gdb --symbols $(DEBUG_OUT)


.PHONY: clean
clean:
	cargo clean