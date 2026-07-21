prog := rampon_anchor
release := --release
target := thumbv6m-none-eabi
extension := uf2

build-kusba:
	cargo build $(release) --target $(target) --no-default-features --features kusba --target-dir target/kusba

build-kusba-is25lp:
	cargo build $(release) --target $(target) --no-default-features --features kusba,is25lp --target-dir target/kusba-is25lp

build-fpis:
	cargo build $(release) --target $(target) --no-default-features --features fpis --target-dir target/fpis

build-mnadxl:
	cargo build $(release) --target $(target) --no-default-features --features mnadxl --target-dir target/mnadxl

binary-kusba:
	elf2uf2-rs target/kusba/$(target)/release/$(prog) release/$(prog)_kusba.$(extension)
	md5sum release/$(prog)_kusba.$(extension) > release/$(prog)_kusba.md5

binary-kusba-is25lp:
	elf2uf2-rs target/kusba-is25lp/$(target)/release/$(prog) release/$(prog)_kusba_is25lp.$(extension)
	md5sum release/$(prog)_kusba_is25lp.$(extension) > release/$(prog)_kusba_is25lp.md5

binary-fpis:
	elf2uf2-rs target/fpis/$(target)/release/$(prog) release/$(prog)_fpis.$(extension)
	md5sum release/$(prog)_fpis.$(extension) > release/$(prog)_fpis.md5

binary-mnadxl:
	elf2uf2-rs target/mnadxl/$(target)/release/$(prog) release/$(prog)_mnadxl.$(extension)
	md5sum release/$(prog)_mnadxl.$(extension) > release/$(prog)_mnadxl.md5

all: build-kusba build-kusba-is25lp build-fpis build-mnadxl binary-kusba binary-kusba-is25lp binary-fpis binary-mnadxl

help:
	@echo "usage: make"
