prog := rampon_anchor
release := --release
target := thumbv6m-none-eabi
extension := uf2

build-kusba:
	cargo build $(release) --target $(target) --no-default-features --features kusba --target-dir target/kusba

build-fpis:
	cargo build $(release) --target $(target) --no-default-features --features fpis --target-dir target/fpis

binary-kusba:
	elf2uf2-rs target/kusba/$(target)/release/$(prog) release/$(prog)_kusba.$(extension)
	md5sum release/$(prog)_kusba.$(extension) > release/$(prog)_kusba.md5

binary-fpis:
	elf2uf2-rs target/fpis/$(target)/release/$(prog) release/$(prog)_fpis.$(extension)
	md5sum release/$(prog)_fpis.$(extension) > release/$(prog)_fpis.md5

all: build-kusba build-fpis binary-kusba binary-fpis

help:
	@echo "usage: make"
