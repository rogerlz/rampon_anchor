use crate::adxl345::Adxl;
use crate::clock::Clock;
use crate::pac::SPI0;

use rp2040_hal::{
    gpio::{bank0::Gpio1, Output, Pin, PushPull},
    spi::{Enabled, Spi},
};

pub struct State {
    pub clock: Clock,
    pub config_crc: Option<u32>,
    pub adxl: Adxl<Spi<Enabled, SPI0, 8>, Pin<Gpio1, Output<PushPull>>>,
}

impl State {
    pub fn poll(&mut self) {}
}
