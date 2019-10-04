extern crate byteorder;
extern crate cc1101;
extern crate linux_embedded_hal as hal;

use byteorder::{BigEndian, ReadBytesExt};

use hal::spidev::SpidevOptions;
use hal::spidev::SpiModeFlags;
use hal::sysfs_gpio::Direction;

use hal::{Pin, Spidev};

use std::{thread, time};

use cc1101::{AddressFilter, Cc1101, Modulation, PacketLength, RadioMode, SyncMode};

mod iterreader;

type RadioErr = cc1101::Error<std::io::Error, hal::sysfs_gpio::Error>;

fn configure_radio(spi: Spidev, cs: Pin) -> Result<Cc1101<Spidev, Pin>, RadioErr> {
    let mut cc1101 = Cc1101::new(spi, cs)?;

    cc1101.set_defaults()?;
    cc1101.set_frequency(868_000_000u64)?;
    cc1101.set_modulation(Modulation::BinaryFrequencyShiftKeying)?;
    cc1101.set_sync_mode(SyncMode::MatchFull(0xD201))?;
    cc1101.set_packet_length(PacketLength::Variable(17))?;
    cc1101.set_address_filter(AddressFilter::Device(0x3e))?;
    cc1101.set_deviation(20629)?;
    cc1101.set_data_rate(38383)?;
    cc1101.set_chanbw(101562)?;

    Ok(cc1101)
}

fn receive_packet(cc1101: &mut Cc1101<Spidev, Pin>) -> Result<(), RadioErr> {
    cc1101.set_radio_mode(RadioMode::Receive)?;

    thread::sleep(time::Duration::from_millis(10));

    let mut dst = 0u8;
    let mut payload = [0u8; 17];

    let length = cc1101.receive(&mut dst, &mut payload)?;
    let rssi = cc1101.get_rssi_dbm()?;
    let lqi = cc1101.get_lqi()?;

    let mut dec = iterreader::IterReader(
        payload[1..]
            .iter()
            .zip([0x47, 0xd0, 0xa2, 0x73, 0x80].iter().cycle())
            .map(|(p, k)| p ^ k),
    );

    let seq = payload[0];
    let status = dec.read_u16::<BigEndian>().unwrap();
    let fixed = dec.read_u32::<BigEndian>().unwrap();
    let pcnt = dec.read_u16::<BigEndian>().unwrap();
    let avg = dec.read_u16::<BigEndian>().unwrap();
    let count = dec.read_u32::<BigEndian>().unwrap();
    let unknown = dec.read_u8().unwrap();

    println!(
        " {:02x} {:02x} {:02x}  {:04x}   {:08x} {:04x} {:04x}    {:08x} {:02x} {:4?} {:3?} # Current power: {}",
        length,
        dst,
        seq,
        status,
        fixed,
        pcnt,
        avg,
        count,
        unknown,
        rssi,
        lqi,
        3686400 / avg as u32
    );

    Ok(())
}

fn main() -> Result<(), RadioErr> {
    let mut spi = Spidev::open("/dev/spidev0.0").expect("Could not open SPI device");
    let options = SpidevOptions::new()
        .max_speed_hz(50_000)
        .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS)
        .build();
    spi.configure(&options).expect("SPI configure error");

    let cs = Pin::new(8);
    cs.export().unwrap();
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).unwrap();

    let mut cc1101 = configure_radio(spi, cs)?;

    println!("Len ID Cnt Status Fixed    PCnt AvgTime PulseCnt ?? RSSI LQI");
    println!("--- -- --- ------ -----    ---- ------- -------- -- ---- ---");

    loop {
        if let Err(err) = receive_packet(&mut cc1101) {
            println!("Error: {:?}", err);
        }
    }
}
