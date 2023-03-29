prog := rampon_anchor
release := --release
target := thumbv6m-none-eabi
extension := uf2

build:
	cargo build $(release) --target $(target)

binary:
	elf2uf2-rs target/$(target)/release/$(prog) release/$(prog).$(extension)
	md5sum release/$(prog).$(extension) > release/$(prog).md5

all: build binary

help:
	@echo "usage: make"
