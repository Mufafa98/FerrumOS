# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

override IMAGE_NAME := ferrum_os
# OLD sdl / gtk
# VGA options cirrus / std / vmware / qxl  
# Resolution -g 
override DISPLAY_TECH := -display gtk -vga vmware 
# Needed for serial output to work.
override DEBUG_PARAMS := -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio 

override INTERRUPT_PARAMS := -d int -D qemu_interrupts.log

override CPU_PARAMS := # -smp 4 

override DISK_PARAMS := -drive file=di_128M.img,format=raw,index=0,id=MFDrive,media=disk

override CUSTOM_PARAMS := $(DISPLAY_TECH) $(DEBUG_PARAMS) $(CPU_PARAMS) $(DISK_PARAMS) 

override Q35_MACHINE := q35
override PC_MACHINE := pc
override MACHINE := -M $(PC_MACHINE)

MODE := 
ifeq ($(MODE), d)
    RFLAGS = debug
else
	RFLAGS = release
endif

.SILENT: run check ovmf limine kernel $(IMAGE_NAME).iso $(IMAGE_NAME).hdd clean distclean

.PHONY: run
run: $(IMAGE_NAME).iso
	qemu-system-x86_64 $(MACHINE) -m 2G -cdrom $(IMAGE_NAME).iso -boot d $(CUSTOM_PARAMS)

.PHONY: check
check:
	$(MAKE) -C kernel check

.PHONY: doc
doc:
	$(MAKE) -C kernel doc
	
ovmf:
	mkdir -p ovmf
	cd ovmf && curl -Lo OVMF-X64.zip https://efi.akeo.ie/OVMF/OVMF-X64.zip && unzip OVMF-X64.zip

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v4.x-branch-binary --depth=1
	$(MAKE) -C limine

.PHONY: kernel
kernel:
	$(MAKE) -C kernel $(RFLAGS)

$(IMAGE_NAME).iso: limine kernel
	rm -rf iso_root
	mkdir -p iso_root
	cp kernel/kernel.elf \
		limine.cfg limine/limine.sys limine/limine-cd.bin limine/limine-cd-efi.bin iso_root/
	xorriso -as mkisofs -b limine-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-cd-efi.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso
	limine/limine-deploy $(IMAGE_NAME).iso
	# rm -rf iso_root

.PHONY: clean
clean:
	rm -rf iso_root $(IMAGE_NAME).iso $(IMAGE_NAME).hdd
	$(MAKE) -C kernel clean

.PHONY: distclean
distclean: clean
	rm -rf limine ovmf
	$(MAKE) -C kernel distclean
