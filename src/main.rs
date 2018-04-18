extern crate cc1101;
extern crate linux_embedded_hal as hal;

use hal::{Pin, Spidev};
use hal::spidev::SpidevOptions;
use hal::sysfs_gpio::Direction;

use std::{thread, time};
use std::fmt::Write;

use cc1101::{Cc1101, Modulation, PacketMode, RadioMode};

fn dump(buf: &[u8]) -> String {
    let mut s = String::new();
    for &byte in buf.iter() {
        write!(&mut s, "{:02x}", byte).expect("Unable to write");
    }
    s
}

fn main() {
    let mut spi = Spidev::open("/dev/spidev0.0").expect("SPI open error");
    let options = SpidevOptions::new().max_speed_hz(50_000).build();
    spi.configure(&options).expect("SPI configure error");

    let cs = Pin::new(24);
    cs.export().unwrap();
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).unwrap();

    let mut cc1101 = Cc1101::new(spi, cs).unwrap();

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
        .set_packet_length(0x11)
        .expect("Setting packet length failed");

    loop {
        // Should use GDO2 to check packet status, then there will be no
        // underflows and radio will remain in receive mode.

        cc1101
            .set_radio_mode(RadioMode::Receive)
            .expect("Enabling reception failed");

        thread::sleep(time::Duration::from_millis(10));

        let mut frame = [0u8; 0x21];
        let mut rssi = 0u8;
        let mut lqi = 0u8;
        cc1101.receive(&mut frame, &mut rssi, &mut lqi).unwrap();

        let buf = &frame[1..];
        let len = buf[0];
        let addr = buf[1];

        if addr != 0x3e {
            continue;
        }

        let payload = &buf[2..18];
        let crc = (buf[18] as u16) << 8 | buf[19] as u16;

        println!(
            "len: {:02x} addr: {:02x} data: {} crc: {:04x} len: {}, ok: {}, {:02x} {:02x}",
            len,
            addr,
            dump(payload),
            crc,
            payload.len(),
            (lqi & 0b10000000) > 0,
            rssi,
            lqi,
        );
    }
}
