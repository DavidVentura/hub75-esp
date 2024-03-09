.PHONY: flash flashm build dump_section_size dump_flash_rodata dump_flash_text
flash:
	cargo espflash flash --release -p /dev/ttyUSB0 -f 80mhz -b 921600
flashm:
	cargo espflash flash --release -p /dev/ttyUSB0 -f 80mhz -b 921600 --flash-mode dio -M
build:
	cargo build --release
