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

.SILENT: all all-hdd run run-uefi run-hdd run-hdd2 run-hdd-uefi check ovmf limine kernel $(IMAGE_NAME).iso $(IMAGE_NAME).hdd clean distclean

.PHONY: all
all: $(IMAGE_NAME).iso

.PHONY: all-hdd
all-hdd: $(IMAGE_NAME).hdd

.PHONY: run
run: $(IMAGE_NAME).iso
	qemu-system-x86_64 $(MACHINE) -m 2G -cdrom $(IMAGE_NAME).iso -boot d $(CUSTOM_PARAMS)

.PHONY: run-hdd
run-hdd: $(IMAGE_NAME).hdd
	qemu-system-x86_64 $(MACHINE) -m 2G -hda $(IMAGE_NAME).hdd $(CUSTOM_PARAMS)

.PHONY: check
check:
	$(MAKE) -C kernel check
.PHONY: doc
doc:
	$(MAKE) -C kernel doc

.PHONY: run-test
run-test: $(IMAGE_NAME).iso.test
	qemu-system-x86_64 $(MACHINE) -m 2G -cdrom $(IMAGE_NAME).iso -boot d $(CUSTOM_PARAMS)

$(IMAGE_NAME).iso.test: limine test
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
	rm -rf iso_root

.PHONY: test
test:
	$(MAKE) -C kernel test

ovmf:
	mkdir -p ovmf
	cd ovmf && curl -Lo OVMF-X64.zip https://efi.akeo.ie/OVMF/OVMF-X64.zip && unzip OVMF-X64.zip

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v4.x-branch-binary --depth=1
	$(MAKE) -C limine

.PHONY: kernel
kernel:
	$(MAKE) -C kernel

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

$(IMAGE_NAME).hdd: limine kernel
	rm -f $(IMAGE_NAME).hdd
	dd if=/dev/zero bs=1M count=0 seek=64 of=$(IMAGE_NAME).hdd
	parted -s $(IMAGE_NAME).hdd mklabel gpt
	parted -s $(IMAGE_NAME).hdd mkpart ESP fat32 2048s 100%
	parted -s $(IMAGE_NAME).hdd set 1 esp on
	limine/limine-deploy $(IMAGE_NAME).hdd
	sudo losetup -Pf --show $(IMAGE_NAME).hdd >loopback_dev
	sudo mkfs.fat -F 32 `cat loopback_dev`p1
	mkdir -p img_mount
	sudo mount `cat loopback_dev`p1 img_mount
	sudo mkdir -p img_mount/EFI/BOOT
	sudo cp -v kernel/kernel.elf limine.cfg limine/limine.sys img_mount/
	sudo cp -v limine/BOOTX64.EFI img_mount/EFI/BOOT/
	sync
	sudo umount img_mount
	sudo losetup -d `cat loopback_dev`
	rm -rf loopback_dev img_mount

.PHONY: clean
clean:
	rm -rf iso_root $(IMAGE_NAME).iso $(IMAGE_NAME).hdd
	$(MAKE) -C kernel clean

.PHONY: distclean
distclean: clean
	rm -rf limine ovmf
	$(MAKE) -C kernel distclean
