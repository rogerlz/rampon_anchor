use crate::adxl345::Adxl;
use crate::clock::Clock;
use crate::pac::SPI1;

use rp2040_hal::{
    gpio::{bank0::Gpio13, Output, Pin, PushPull},
    spi::{Enabled, Spi},
};

pub struct State {
    pub clock: Clock,
    pub config_crc: Option<u32>,
    pub adxl: Adxl<Spi<Enabled, SPI1, 8>, Pin<Gpio13, Output<PushPull>>>,
}

impl State {
    pub fn poll(&mut self) {}
}
