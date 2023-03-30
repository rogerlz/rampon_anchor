# Rampon Anchor Firmware

Rampon is an alternate firmware you can flash on the [KUSBA V2],
or any Pico or RP2040 (like the [Seeed
Studio Xiao][xiao] or the [Adafruit QT Py][qtpy]) with an
ADXL345 attached. 

You can use Rampon with any of these devices to run input shaper
calibration for Klipper. Rampon currently requires the ADXL345 to
be wired to the same GPIO pins that the KUSBA V2 uses - GPIO0,
GPIO1, GPIO2, GPIO3.

At the moment, the SPI0 bus is usable _only_ via these pins. If
you used different SPI0 pins, wired the ADXL345 to SPI1 pins
instead, or wired a second ADXL345 to your RP2040, the current
version of Rampon will not work with these ADXL345s.

## Loading Release Builds

1. Download the UF2 file from the release assets
2. Hold the BOOT or BOOTSEL button on the PCB while connecting USB cable
3. Mount the USB storage device if necessary
4. Copy the UF2 file manually or run:  
``` 
% sudo elf2uf2-rs -d rampon_anchor.uf2
```

After flashing completes, the device should be available as:
`/dev/serial/by-id/usb-Anchor_Rampon-if00` when plugged into
a Linux machine.

## Klipper Config

```ini
[mcu rampon]
serial: /dev/serial/by-id/usb-Anchor_Rampon-if00

[adxl345]
cs_pin: rampon:CS

[resonance_tester]
accel_chip: adxl345
probe_points: 90, 90, 20
```

## Developers

---

## Building Firmware

To compile the project, you will need a Rust toolchain installed, `cargo-binutils`, and the compile target for ARM Cortex-M0. They can be installed with:

```
% rustup component add llvm-tools-preview
% rustup target add thumbv6m-none-eabi
```

To build the project run:

```
% cargo build --release --target thumbv6m-none-eabi
```

To flash an RP2040 connected over USB in bootloader mode, run:

```
% sudo elf2uf2-rs -d target/thumbv6m-none-eabi/release/rampon_anchor
```

After the update completes, the device should be available as:
`/dev/serial/by-id/usb-Anchor_Rampon-if00`.

# Credits / Related Projects

Rampon uses code from [Annex Engineering]'s [crampon_anchor] and
[Anchor] projects (specifically, the [rp2040_demo]).

[KUSBA V2]: <https://github.com/xbst/KUSBA>
[xiao]: <https://wiki.seeedstudio.com/XIAO-RP2040/>
[qtpy]: <https://learn.adafruit.com/adafruit-qt-py-2040>
[Annex Engineering]: <https://github.com/Annex-Engineering>
[crampon_anchor]: <https://github.com/Annex-Engineering/crampon_anchor>
[Anchor]: <https://github.com/Annex-Engineering/anchor>
[rp2040_demo]: <https://github.com/Annex-Engineering/anchor/tree/master/rp2040_demo>