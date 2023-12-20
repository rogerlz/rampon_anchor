#![no_std]
#![no_main]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]

#[cfg(not(any(feature = "kusba", feature = "fpis", feature = "mnadxl")))]
compile_error!("One of `kusba` or `fpis` or `mnadxl` must be enabled.");

#[cfg(all(feature = "kusba", feature = "fpis", feature = "mnadxl"))]
compile_error!("Only one of `kusba` or `fpis` or `mnadxl` can be enabled.");

mod adxl345;
mod clock;
mod commands;
mod usb;

use rp_pico as bsp;

use fugit::RateExtU32;
use panic_halt as _;

use bsp::{
    entry,
    hal::{clocks::init_clocks_and_plls, pac, sio::Sio, usb::UsbBus, watchdog::Watchdog},
};
use cortex_m::interrupt::free;
use embedded_hal::spi::MODE_3;

use rp2040_hal::{
    gpio::{FunctionSpi, Pins},
    spi::Spi,
    Clock,
};

use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::{CdcAcmClass, USB_CLASS_CDC};

use anchor::*;
use usb::*;

#[cfg(feature = "kusba")]
mod kusba;

#[cfg(feature = "fpis")]
mod fpis;

#[cfg(feature = "mnadxl")]
mod mnadxl;

#[cfg(feature = "kusba")]
use kusba::State;

#[cfg(feature = "fpis")]
use fpis::State;

#[cfg(feature = "mnadxl")]
use mnadxl::State;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let sio = Sio::new(pac.SIO);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = bsp::XOSC_CRYSTAL_FREQ;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // USB
    let usb_allocator = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Serial
    let mut serial = CdcAcmClass::new(&usb_allocator, 64);
    let mut bus = UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(0x1d50, 0x614e))
        .composite_with_iads()
        .manufacturer("Anchor")
        .product("Rampon")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut read_buffer = FifoBuffer::<128>::new();
    let mut packet_writer = UsbPacketWriter::default();

    // SPI
    #[cfg(feature = "kusba")]
    let (spi, spi_cs) = {
        let _sclk = pins.gpio2.into_mode::<FunctionSpi>();
        let _mosi = pins.gpio3.into_mode::<FunctionSpi>();
        let _miso = pins.gpio0.into_mode::<FunctionSpi>();
        let cs = pins.gpio1.into_push_pull_output();

        let spi = Spi::<_, _, 8>::new(pac.SPI0).init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            8.MHz(),
            &MODE_3,
        );

        (spi, cs)
    };

    #[cfg(feature = "fpis")]
    let (spi, spi_cs) = {
        let _sclk = pins.gpio10.into_mode::<FunctionSpi>();
        let _mosi = pins.gpio11.into_mode::<FunctionSpi>();
        let _miso = pins.gpio12.into_mode::<FunctionSpi>();
        let cs = pins.gpio13.into_push_pull_output();

        let spi = Spi::<_, _, 8>::new(pac.SPI1).init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            8.MHz(),
            &MODE_3,
        );

        (spi, cs)
    };

    #[cfg(feature = "mnadxl")]
    let (spi, spi_cs) = {
        let _sclk = pins.gpio10.into_mode::<FunctionSpi>();
        let _mosi = pins.gpio11.into_mode::<FunctionSpi>();
        let _miso = pins.gpio12.into_mode::<FunctionSpi>();
        let cs = pins.gpio9.into_push_pull_output();

        let spi = Spi::<_, _, 8>::new(pac.SPI1).init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            8.MHz(),
            &MODE_3,
        );

        (spi, cs)
    };

    let adxl = adxl345::Adxl::init(spi, spi_cs);

    let mut state = State {
        clock: clock::Clock::new(pac.TIMER),
        config_crc: None,
        adxl: adxl,
    };

    // Run Forever
    loop {
        state.poll();

        // Read side
        bus.poll(&mut [&mut serial]);
        while let Ok(n) = serial.read_packet(read_buffer.receive_buffer()) {
            read_buffer.advance(n);
        }
        if !read_buffer.is_empty() {
            let mut wrap = SliceInputBuffer::new(read_buffer.data());
            KLIPPER_TRANSPORT.receive(&mut wrap, &mut state);
            read_buffer.pop(read_buffer.len() - wrap.available());
        }

        // Write side
        free(|cs| {
            let mut txbuf = USB_TX_BUFFER.borrow(cs).borrow_mut();
            packet_writer.write_packets(&mut serial, &mut txbuf);
        });
        bus.poll(&mut [&mut serial]);

        state.adxl.run(state.clock.low());
    }
}

klipper_config_generate!(
    transport = crate::usb::TRANSPORT_OUTPUT: crate::usb::BufferTransportOutput,
    context = &'ctx mut crate::State,
);

#[klipper_constant]
const MCU: &str = "rampon_anchor";

#[klipper_constant]
const STATS_SUMSQ_BASE: u32 = 256;

klipper_enumeration!(
    enum spi_bus {
        spi0,
        spi1,
    }
);

klipper_enumeration!(
    enum pin {
        CS,
    }
);
