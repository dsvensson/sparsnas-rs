extern crate byteorder;
extern crate cc1101;

use byteorder::{BigEndian, ReadBytesExt};

use std::{thread, time};

use cc1101::{AddressFilter, Cc1101, Modulation, PacketLength, RadioMode, SyncMode};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use rppal::gpio::{Gpio, OutputPin};

mod iterreader;

type RadioErr = cc1101::Error<rppal::spi::Error, ()>;

fn configure_radio(spi: Spi, cs: OutputPin) -> Result<Cc1101<Spi, OutputPin>, RadioErr> {
    let mut cc1101 = Cc1101::new(spi, cs)?;

    cc1101.set_defaults()?;
    cc1101.set_frequency(868_000_000u64)?;
    cc1101.set_modulation(Modulation::BinaryFrequencyShiftKeying)?;
    cc1101.set_sync_mode(SyncMode::MatchFull(0xD201))?;
    cc1101.set_packet_length(PacketLength::Variable(17))?;
    cc1101.set_address_filter(AddressFilter::Device(0x3e))?;
    cc1101.set_deviation(20_629)?;
    cc1101.set_data_rate(38_383)?;
    cc1101.set_chanbw(101_562)?;

    Ok(cc1101)
}

fn receive_packet(cc1101: &mut Cc1101<Spi, OutputPin>) -> Result<(), RadioErr> {
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
        3_686_400 / u32::from(avg)
    );

    Ok(())
}

fn main() -> Result<(), RadioErr> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 50_000, Mode::Mode0).unwrap();
    let cs = Gpio::new().unwrap().get(8).unwrap().into_output();

    let mut cc1101 = configure_radio(spi, cs)?;

    println!("Len ID Cnt Status Fixed    PCnt AvgTime PulseCnt ?? RSSI LQI");
    println!("--- -- --- ------ -----    ---- ------- -------- -- ---- ---");

    loop {
        if let Err(err) = receive_packet(&mut cc1101) {
            println!("Error: {:?}", err);
        }
    }

}
