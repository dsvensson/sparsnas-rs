extern crate byteorder;
extern crate cc1101;
extern crate linux_embedded_hal as hal;

use hal::{Pin, Spidev};
use hal::spidev::SpidevOptions;
use hal::sysfs_gpio::Direction;
use byteorder::*;

use std::{thread, time};
use std::fmt::Write;
use std::io::Cursor;

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
        .set_packet_length(20)
        .expect("Setting packet length failed");

    loop {
        // Should use GDO2 to check packet status, then there will be no
        // underflows and radio will remain in receive mode.

        cc1101
            .set_radio_mode(RadioMode::Receive)
            .expect("Enabling reception failed");

        thread::sleep(time::Duration::from_millis(10));

        let mut frame = [0u8; 21];
        let mut rssi = 0u8;
        let mut lqi = 0u8;
        cc1101.receive(&mut frame, &mut rssi, &mut lqi).unwrap();

        let buf = &frame[1..];
        let len = buf[0];
        let addr = buf[1];

        if len != 0x11 && addr != 0x3e {
            continue;
        }

        let seq = buf[2];
        let payload = &buf[3..18];

        /*
        println!(
            "len: {:02x} addr: {:02x} data: {} len: {}, ok: {}, {:02x} {:02x}",
            len,
            addr,
            dump(payload),
            payload.len(),
            (lqi & 0b10000000) > 0,
            rssi,
            lqi,
        );
         */

        let mut dec = payload
            .iter()
            .zip([0x47, 0xd0, 0xa2, 0x73, 0x80].iter().cycle())
            .map(|(p, k)| p ^ k);

        let mut r = Cursor::new(dec);

        println!("{}", r.read_u16::<BigEndian>().unwrap());

        /*
Error:
        pi@raspberrypi:~/sparsnas $ cargo run
        Compiling sparsnas v0.1.0 (file:///home/pi/sparsnas)
        error[E0599]: no method named `read_u16` found for type `std::io::Cursor<std::iter::Map<std::iter::Zip<std::slice::Iter<'_, u8>, std::iter::Cycle<std::slice::Iter<'_, u8>>>, [closure@src/main.rs:106:18: 106:32]>>` in the current scope
        --> src/main.rs:110:26
        |
        110 |         println!("{}", r.read_u16::<BigEndian>().unwrap());
        |                          ^^^^^^^^
        |
        = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
        = help: try with `std::io::Cursor<std::iter::Map<std::iter::Zip<std::slice::Iter<'_, u8>, std::iter::Cycle<std::slice::Iter<'_, u8>>>, [closure@src/main.rs:106:18: 106:32]>>::read_u16`
        note: candidate #1 is defined in the trait `byteorder::ByteOrder`
        --> /home/pi/.cargo/registry/src/github.com-1ecc6299db9ec823/byteorder-1.2.2/src/lib.rs:221:5
        |
        221 |     fn read_u16(buf: &[u8]) -> u16;
        |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        = help: to disambiguate the method call, write `byteorder::ByteOrder::read_u16(r)` instead
        = note: the method `read_u16` exists but the following trait bounds were not satisfied:
        `std::io::Cursor<std::iter::Map<std::iter::Zip<std::slice::Iter<'_, u8>, std::iter::Cycle<std::slice::Iter<'_, u8>>>, [closure@src/main.rs:106:18: 106:32]>> : byteorder::ReadBytesExt`

        error: aborting due to previous error

        For more information about this error, try `rustc --explain E0599`.
        error: Could not compile `sparsnas`.

        To learn more, run the command again with --verbose.
*/

        /*
        let status = ((dec.next().unwrap() as u16) << 8) | dec.next().unwrap() as u16;
        let fixed = ((dec.next().unwrap() as u32) << 24) | ((dec.next().unwrap() as u32) << 16)
            | ((dec.next().unwrap() as u32) << 8) | dec.next().unwrap() as u32;
        let pcnt = ((dec.next().unwrap() as u16) << 8) | dec.next().unwrap() as u16;
        let avg = ((dec.next().unwrap() as u16) << 8) | dec.next().unwrap() as u16;
        let count = ((dec.next().unwrap() as u32) << 24) | ((dec.next().unwrap() as u32) << 16)
            | ((dec.next().unwrap() as u32) << 8) | dec.next().unwrap() as u32;
        let unknown = dec.next().unwrap();

        println!(
            " {:02x} {:02x} {:02x}  {:04x}   {:08x} {:04x} {:04x}    {:08x} {:02x} # Current power: {}",
            len, addr, seq, status, fixed, pcnt, avg, count, unknown, 3686400 / avg as u32
        );
*/
    }
}
