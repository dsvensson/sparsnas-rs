extern crate byteorder;
extern crate cc1101;
extern crate hex;
extern crate linux_embedded_hal as hal;

use byteorder::{BigEndian, ReadBytesExt};

use hal::spidev::SpidevOptions;
use hal::sysfs_gpio::Direction;
use hal::{Pin, Spidev};

use std::io::{Error, Read};
use std::{thread, time};

use cc1101::{Cc1101, Modulation, PacketMode, RadioMode};

struct IterReader<I: Iterator<Item = u8>>(I);

impl<'a, I: Iterator<Item = u8>> Read for IterReader<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut count: usize = 0;
        for i in 0..buf.len() {
            match self.0.next() {
                Some(v) => {
                    buf[i] = v;
                    count += 1;
                }
                None => break,
            }
        }
        Ok(count)
    }
}

fn configure_radio(spi: Spidev, cs: Pin) -> Result<Cc1101<Spidev, Pin>, Error> {
    let mut cc1101 = Cc1101::new(spi, cs)?;

    cc1101.reset().expect("Reset failed");

    cc1101
        .set_defaults()
        .expect("Setting default values failed");

    cc1101
        .set_sync_word(0xD201)
        .expect("Setting sync word failed");

    cc1101.set_sync_mode(1).expect("Setting sync mode failed");

    cc1101
        .set_modulation(Modulation::MOD_2FSK)
        .expect("Setting sync mode failed");

    cc1101
        .set_frequency(868_000_000u64)
        .expect("Setting frequency failed");

    cc1101
        .set_packet_mode(PacketMode::Fixed)
        .expect("Setting packet mode failed");

    cc1101
        .set_packet_length(20)
        .expect("Setting packet length failed");

    Ok(cc1101)
}

fn receive_packet(cc1101: &mut Cc1101<Spidev, Pin>) -> Result<(), Error> {
    cc1101
        .set_radio_mode(RadioMode::Receive)
        .expect("Enabling reception failed");

    thread::sleep(time::Duration::from_millis(10));

    let mut frame = [0u8; 21];
    let mut rssi = 0u8;
    let mut lqi = 0u8;

    cc1101.receive(&mut frame, &mut rssi, &mut lqi)?;

    let buf = &frame[1..];
    let len = buf[0];
    let addr = buf[1];

    let seq = buf[2];
    let payload = &buf[3..18];

    println!(
        "len: {:02x} addr: {:02x} data: {} len: {}, ok: {}, {:02x} {:02x}",
        len,
        addr,
        hex::encode(payload),
        payload.len(),
        (lqi & 0b10000000) > 0,
        rssi,
        lqi,
    );

    // Should fix CRC again... and probably check some package characteristics.
    if len == 0x11 && addr == 0x3e {
        let mut dec = IterReader(
            payload
                .iter()
                .zip([0x47, 0xd0, 0xa2, 0x73, 0x80].iter().cycle())
                .map(|(p, k)| p ^ k),
        );

        let status = dec.read_u16::<BigEndian>().unwrap();
        let fixed = dec.read_u32::<BigEndian>().unwrap();
        let pcnt = dec.read_u16::<BigEndian>().unwrap();
        let avg = dec.read_u16::<BigEndian>().unwrap();
        let count = dec.read_u32::<BigEndian>().unwrap();
        let unknown = dec.read_u8().unwrap();

        println!(
            " {:02x} {:02x} {:02x}  {:04x}   {:08x} {:04x} {:04x}    {:08x} {:02x} # Current power: {}",
            len, addr, seq, status, fixed, pcnt, avg, count, unknown, 3686400 / avg as u32
        );
    }

    Ok(())
}

fn run() -> Result<(), Error> {
    let mut spi = Spidev::open("/dev/spidev0.0").expect("Could not open SPI device");
    let options = SpidevOptions::new().max_speed_hz(50_000).build();
    spi.configure(&options).expect("SPI configure error");

    let cs = Pin::new(24);
    cs.export().unwrap();
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).unwrap();

    let mut cc1101 = configure_radio(spi, cs)?;

    loop {
        receive_packet(&mut cc1101)?;
    }
}

fn main() {
    run().unwrap();
}
