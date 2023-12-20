# Rampon Anchor Firmware

Rampon is an alternative firmware that you can flash on various devices including the
[KUSBA V2], [FYSETC PortableInputShaper][PIS], [SeeedStudio Xiao][xiao],
[Adafruit QT Py][qtpy], [Mellow Nozzle ADXL][MNADXL], or any Pico or RP2040 with an
ADXL345 attached.

You can use Rampon to run input shaper calibration for Klipper with any of these devices.
However, there are two different versions of Rampon available depending on which device you are using.

 - If you are using the KUSBA V2 or any other device wired to SPI0, you can use the `kusba`
version of Rampon.

 - If you are using the FYSETC PortableInputShaper or any other device wired
to SPI1 and CS to gpio13, you need to use the `fpis` version of Rampon.

 - If you are using the Mellow Nozzle ADXL or any other device wired
to SPI1 and CS to gpio9, you need to use the `mnadxl` version of Rampon.

They are available as separate binaries.

It's important to note that the current version of Rampon requires the ADXL345 to be wired to
specific GPIO pins depending on which version of Rampon you are using.

  - If you are using the `kusba` version of Rampon for SPI0, the ADXL345 needs to
  be wired to GPIO0, GPIO1, GPIO2, and GPIO3.

  - If you are using the `fpis` version of Rampon for SPI1, the ADXL345 needs to
  be wired to GPIO10, GPIO11, GPIO12, and GPIO13.

  - If you are using the `mnadxl` version of Rampon for SPI1, the ADXL345 needs to
  be wired to GPIO10, GPIO11, GPIO12, and GPIO13.

If you have wired the ADXL345 differently or have wired a second ADXL345 to your RP2040,
the current version of Rampon might not work for you.

Note: The FYSETC PortableInputShaper has an LED that can be used to indicate the status
of the input shaper. However, the LED functionality has not been implemented in Rampon yet.

## Loading Release Builds

1. Download the UF2 files from the release assets, choosing the correct version depending
the device you have.
2. Hold the BOOT or BOOTSEL button on the PCB while connecting USB cable
3. Mount the USB storage device if necessary
4. Copy the UF2 file manually or run:

```
% sudo elf2uf2-rs -d rampon_anchor_VERSION.uf2
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
% cargo build --release --target thumbv6m-none-eabi --no-default-features --features kusba
```

To flash an RP2040 connected over USB in bootloader mode, run:

```
% sudo elf2uf2-rs -d target/thumbv6m-none-eabi/release/rampon_anchor
```

After the update completes, the device should be available as:
`/dev/serial/by-id/usb-Anchor_Rampon-if00`.

## Available Rust Features

 - kusba
 - fpis
 - mnadxl

## Release

To create the release files

```
% make all
```

# Credits / Related Projects

Rampon uses code from [Annex Engineering]'s [crampon_anchor] and
[Anchor] projects (specifically, the [rp2040_demo]).

[KUSBA V2]: https://github.com/xbst/KUSBA
[PIS]: https://github.com/FYSETC/FYSETC-PortableInputShaper
[MNADXL]: https://github.com/Mellow-3D/USB-Accelerometer
[xiao]: https://wiki.seeedstudio.com/XIAO-RP2040/
[qtpy]: https://learn.adafruit.com/adafruit-qt-py-2040
[Annex Engineering]: https://github.com/Annex-Engineering
[crampon_anchor]: https://github.com/Annex-Engineering/crampon_anchor
[Anchor]: https://github.com/Annex-Engineering/anchor
[rp2040_demo]: https://github.com/Annex-Engineering/anchor/tree/master/rp2040_demo

Also thanks to [Clee](https://github.com/clee) for always offering help to test.
