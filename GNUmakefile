# --- Configuration ---
IMAGE_NAME := ferrum_os
DISK_IMAGE := di_128M.img
MODE       ?= release

ifeq ($(MODE), d)
	RFLAGS := debug
else
	RFLAGS := release
endif

# --- QEMU Parameters ---
MACHINE          := -M pc
DISPLAY_TECH     := -display gtk -vga vmware
DEBUG_PARAMS     := -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio
DISK_PARAMS      := -drive file=$(DISK_IMAGE),format=raw,index=0,id=MFDrive,media=disk
INTERRUPT_PARAMS := -d int -D qemu_interrupts.log
CPU_PARAMS       := 
CUSTOM_PARAMS    := $(DISPLAY_TECH) $(DEBUG_PARAMS) $(CPU_PARAMS) $(DISK_PARAMS)

# --- Build Rules ---
.PHONY: all run check doc kernel clean distclean
.SILENT: run check ovmf limine kernel $(IMAGE_NAME).iso clean distclean

all: $(IMAGE_NAME).iso

$(DISK_IMAGE):
	truncate -s 128M $(DISK_IMAGE)
	mkfs.ext2 -F $(DISK_IMAGE)

run: $(IMAGE_NAME).iso $(DISK_IMAGE)
	qemu-system-x86_64 $(MACHINE) -m 2G -cdrom $(IMAGE_NAME).iso -boot d $(CUSTOM_PARAMS)

check:
	$(MAKE) -C kernel check

doc:
	$(MAKE) -C kernel doc

ovmf:
	mkdir -p ovmf
	cd ovmf && curl -Lo OVMF-X64.zip https://efi.akeo.ie/OVMF/OVMF-X64.zip && unzip OVMF-X64.zip

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v4.x-branch-binary --depth=1
	$(MAKE) -C limine

kernel:
	$(MAKE) -C kernel $(RFLAGS)

$(IMAGE_NAME).iso: limine kernel
	rm -rf iso_root
	mkdir -p iso_root
	cp kernel/kernel.elf limine.cfg limine/limine.sys limine/limine-cd.bin limine/limine-cd-efi.bin iso_root/
	xorriso -as mkisofs -b limine-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-cd-efi.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso
	limine/limine-deploy $(IMAGE_NAME).iso

clean:
	rm -rf iso_root $(IMAGE_NAME).iso $(IMAGE_NAME).hdd
	$(MAKE) -C kernel clean

distclean: clean
	rm -rf limine ovmf
	$(MAKE) -C kernel distclean