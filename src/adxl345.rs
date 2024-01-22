use crate::{clock, clock::InstantShort, State};
use anchor::{klipper_command, klipper_reply};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

struct RegAddr;

impl RegAddr {
    const DEVID: u8 = 0;
    const THRESH_TAP: u8 = 29;
    const OFSX: u8 = 30;
    const OFSY: u8 = 31;
    const OFSZ: u8 = 32;
    const DUR: u8 = 33;
    const LATENT: u8 = 34;
    const WINDOW: u8 = 35;
    const THRESH_ACT: u8 = 36;
    const THRESH_INACT: u8 = 37;
    const TIME_INACT: u8 = 38;
    const ACT_INACT_CTL: u8 = 39;
    const THRESH_FF: u8 = 40;
    const TIME_FF: u8 = 41;
    const TAP_AXES: u8 = 42;
    const ACT_TAP_STATUS: u8 = 43;
    const BW_RATE: u8 = 44;
    const POWER_CTL: u8 = 45;
    const INT_ENABLE: u8 = 46;
    const INT_MAP: u8 = 47;
    const INT_SOURCE: u8 = 48;
    const DATA_FORMAT: u8 = 49;
    const DATAX0: u8 = 50;
    const DATAX1: u8 = 51;
    const DATAY0: u8 = 52;
    const DATAY1: u8 = 53;
    const DATAZ0: u8 = 54;
    const DATAZ1: u8 = 55;
    const FIFO_CTL: u8 = 56;
    const FIFO_STATUS: u8 = 57;

    const AF_MULTI: u8 = 0x40;
    const AF_READ: u8 = 0x80;
}

pub struct SampleBuffer<const N: usize> {
    count: usize,
    buffer: [u8; N],
}

pub struct Adxl<SPI, PIN> {
    spi: SPI,
    cs: PIN,
    oid: u8,
    wake_time: Option<InstantShort>,
    rest_ticks: u32,
    sequence: u16,
    limit: u16,
    buffer: SampleBuffer<50>,
}

impl<const N: usize> SampleBuffer<N> {
    pub fn init() -> Self {
        SampleBuffer {
            count: 0,
            buffer: [0; N],
        }
    }

    pub fn full(&self) -> bool {
        self.count + 5 > N
    }

    pub fn empty(&self) -> bool {
        self.count == 0
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn contents(&mut self) -> &[u8] {
        &self.buffer[0..self.count]
    }

    pub fn push(&mut self, d: &[u8]) {
        self.buffer[self.count] = d[1];
        self.buffer[self.count + 1] = d[3];
        self.buffer[self.count + 2] = d[5];
        self.buffer[self.count + 3] = (d[2] & 0x1f) | (d[6] << 5);
        self.buffer[self.count + 4] = (d[4] & 0x1f) | ((d[6] << 2) & 0x60);
        self.count += 5;
    }
}

impl<SPI, PIN> Adxl<SPI, PIN>
where
    SPI: Transfer<u8> + Write<u8>,
    PIN: OutputPin,
{
    pub fn init(spi: SPI, cs: PIN) -> Self {
        let mut adxl = Adxl {
            spi,
            cs,
            oid: 0,
            wake_time: None,
            rest_ticks: 0,
            sequence: 0,
            limit: 0,
            buffer: SampleBuffer::<50>::init(),
        };

        // prime sensor with devid read
        adxl.send(&[RegAddr::AF_READ | RegAddr::DEVID, 0]);

        adxl
    }

    fn start(&mut self, clock: InstantShort, rest_ticks: u32) {
        self.limit = 0;
        self.sequence = 0;
        self.buffer.clear();
        self.rest_ticks = rest_ticks;
        self.wake_time = Some(clock + rest_ticks);
    }

    fn stop(&mut self) {
        self.wake_time = None;
    }

    fn send(&mut self, data: &[u8]) {
        self.cs.set_low().ok();
        self.spi.write(data).ok().unwrap();
        self.cs.set_high().ok();
    }

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> &'w [u8] {
        self.cs.set_low().ok();
        let resp = self.spi.transfer(data).ok().unwrap();
        self.cs.set_high().ok();
        resp
    }

    fn query(&mut self) -> u32 {
        let mut msgo: [u8; 9] = [0; 9];
        let mut wake_ticks = 0;
        msgo[0] = RegAddr::DATAX0 | RegAddr::AF_MULTI | RegAddr::AF_READ;

        self.cs.set_low().ok();
        let msgi = self.spi.transfer(&mut msgo).ok().unwrap();
        self.cs.set_high().ok();
        let mut fifo = msgi[8] & 0x7f;

        if (msgi[2] & 0xf0 > 0 && msgi[2] & 0xf0 != 0xf0)
            || (msgi[4] & 0xf0 > 0 && msgi[4] & 0xf0 != 0xf0)
            || (msgi[6] & 0xf0 > 0 && msgi[6] & 0xf0 != 0xf0)
            || (msgi[7] != 0x90)
            || (fifo > 32)
        {
            self.buffer.push(&[0xff; 7]);
            fifo = 0;
        } else {
            self.buffer.push(msgi);
        }

        if self.buffer.full() {
            self.report();
        }
        if fifo >= 31 {
            self.limit += 1;
        }
        if fifo == 0 {
            wake_ticks = self.rest_ticks;
        }
        wake_ticks
    }

    fn status(&mut self, clock: &clock::Clock) {
        let before = clock.low();
        self.cs.set_low().ok();
        let mut msgo = [RegAddr::FIFO_STATUS | RegAddr::AF_READ, 0];
        let msgi = self.spi.transfer(&mut msgo).ok().unwrap();
        self.cs.set_high().ok();
        let after = clock.low();
        let fifo_packed_len = (msgi[1] & 0x7F) as u32 * 5 as u32;
        self.send_status(before, after, fifo_packed_len);
    }

    fn send_status(&self, before: InstantShort, after: InstantShort, fifo: u32) {
        let delta = u32::from(after).wrapping_sub(u32::from(before));
        klipper_reply!(
            sensor_bulk_status,
            oid: u8 = self.oid,
            clock: u32 = before.into(),
            query_ticks: u32 = delta,
            next_sequence: u16 = self.sequence,
            buffered: u32 = self.buffer.count() as u32 + fifo,
            possible_overflows: u16 = self.limit
        );
    }

    fn report(&mut self) {
        klipper_reply!(
            sensor_bulk_data,
            oid: u8 = self.oid,
            sequence: u16 = self.sequence,
            data: &[u8] = self.buffer.contents()
        );
        self.buffer.clear();
        self.sequence += 1;
    }

    fn sched_wake(&mut self, clock: InstantShort) {
        self.wake_time = Some(clock + self.rest_ticks);
    }

    pub fn run(&mut self, clock: InstantShort) {
        if let Some(wt) = self.wake_time {
            if clock.after(wt) {
                let rest = self.query();
                self.wake_time = Some(clock + rest);
            }
        }
    }
}

#[klipper_command]
pub fn config_adxl345(context: &mut State, oid: u8, _spi_oid: u8) {
    context.adxl.oid = oid;
}

#[klipper_command]
pub fn query_adxl345(context: &mut State, _oid: u8, rest_ticks: u32) {
    if rest_ticks != 0 {
        context.adxl.start(context.clock.low(), rest_ticks);
    } else {
        context.adxl.stop();
    }
}

#[klipper_command]
pub fn query_adxl345_status(context: &mut State, _oid: u8) {
    context.adxl.status(&context.clock);
}

#[klipper_command]
pub fn config_spi_shutdown(_context: &mut State, _oid: u8, _spi_oid: u8, _shutdown_msg: &[u8]) {}

#[klipper_command]
pub fn spi_send(context: &mut State, _oid: u8, data: &[u8]) {
    context.adxl.send(data);
}

#[klipper_command]
pub fn spi_transfer(context: &mut State, oid: u8, data: &[u8]) {
    let mut buffer: [u8; 32] = [0; 32];
    let len = data.len().min(buffer.len());
    buffer[..len].copy_from_slice(&data[..len]);
    let resp = context.adxl.transfer(&mut buffer[..len]);
    klipper_reply!(spi_transfer_response, oid: u8 = oid, response: &[u8] = resp);
}

#[klipper_command]
pub fn config_spi(_context: &mut State, _oid: u8, _pin: u32, _cs_active_high: u8) {}

#[klipper_command]
pub fn config_spi_without_cs(_context: &mut State, _oid: u8) {}

#[klipper_command]
pub fn spi_set_bus(_context: &mut State, _oid: u8, _spi_bus: u32, _mode: u32, _rate: u32) {}
