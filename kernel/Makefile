
ARCH="riscv64gc-unknown-none-elf"
BOARD=qemu
ODIR="target/${ARCH}/release"

BL="../bootloader/riscv64-bootloader.bin"

.PHONY: bin
bin: ${ODIR}/rustos.bin

${ODIR}/rustos:
	cargo build --release --features "board_$(BOARD)"

${ODIR}/rustos.bin: ${ODIR}/rustos
	rust-objcopy --strip-all $< -O binary $@

.PHONY: run_wait
run_wait: ${ODIR}/rustos.bin
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ${BL} \
	-device loader,file=${ODIR}/rustos.bin,addr=0x80200000 \
	-s -S

.PHONY: run
run: ${ODIR}/rustos.bin
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ${BL} \
	-device loader,file=${ODIR}/rustos.bin,addr=0x80200000

.PHONY: gdb
gdb:
	riscv64-elf-gdb \
	-ex 'file ${ODIR}/rustos' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote localhost:1234'
