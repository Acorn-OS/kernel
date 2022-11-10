DEBUG_OUT := target/amd64/debug/kernel
SRC_ROOT = $(abspath .)

.PHONY: all 
all: qemu

.PHONY: debug_build
debug_build:
	@cargo build --target ${SRC_ROOT}/amd64.json
	@cp $(DEBUG_OUT) $(DEBUG_OUT).stripped
	@objcopy $(DEBUG_OUT).stripped -O binary

.PHONY: qemu
qemu: debug_build
	qemu-system-x86_64 -drive format=raw,file=$(DEBUG_OUT).stripped -s -S --no-reboot -m 3G -monitor stdio -d in_asm,int,unimp,mmu,cpu_reset,guest_errors,page -D qemu.log

.PHONY: clean
clean:
	cargo clean