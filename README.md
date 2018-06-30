## IKEA Sparsnäs receiver

Work in progress, should be configurable, and perhaps turned into a library and another
application would do the actual work of storing the data in for example Influx DB. Currently
CRC check doesn't pass in the [cc1101 crate][2], should be fixed too. I'm fairly new with Rust,
so all help, nags, recommendations appreciated.

## Screenshot

    $ cargo run
    Compiling sparsnas v0.1.0 (file:///home/pi/sparsnas)
    Finished dev [unoptimized + debuginfo] target(s) in 6.83 secs
    Running `target/debug/sparsnas`
    11 3e 39  40de   0009693e ed39 3207    000205e3 64 # Current power: 287
    11 3e 3a  40de   0009693e ed3a 30d2    000205e5 64 # Current power: 294
    11 3e 3b  40de   0009693e ed3b 3009    000205e6 64 # Current power: 299
    11 3e 3c  40de   0009693e ed3c 2fd6    000205e7 64 # Current power: 301

## Hardware

A [Ti CC1101][1] (available for about 3-4 USD) connected to a Raspberry Pi:

    Vdd    -    3.3V (P1-17)
    SI     -    MOSI (P1-19)
    SO     -    MISO (P1-21)
    CS     -    SS   (P1-24)
    SCLK   -    SCK  (P1-23)
    GDO2   -    GPIO (P1-22)
    GND    -    GND  (P1-25)

## Raspberry Pi Setup

Update to the latest kernel via `sudo rpi-update`, and add the following SPI
settings to `/boot/config.txt`:

    dtparam=spi=on
    dtoverlay=spi0-hw-cs

The overlay is needed to get `SPI_NO_CS` working. Without this it's not possible to
perform multiple SPI transfers within the same *chip-select*.

## Cross compilation

Compiling this on the Raspberry Pi is fairly painful. To get cross compilation going add
the following to `~/.cargo/config`:

    [target.armv7-unknown-linux-musleabihf]
    linker = "arm-linux-gnueabihf-gcc"

And install the ARM toolchain:

    sudo apt-get install gcc-arm-linux-gnueabihf

The development roundtrip can now be shortened by:

    cargo build --target=armv7-unknown-linux-musleabihf && \
      rsync -vPz target/armv7-unknown-linux-musleabihf/debug/sparsnas pi@raspberrypi.local:/tmp && \
      ssh pi@raspberrypi.local "killall -9 sparsnas; /tmp/sparsnas"

## License

Licensed under your option of:

* [MIT License](LICENSE-MIT)
* [Apache License, Version 2.0](LICENSE-APACHE)


[1]: https://web.archive.org/web/20171202153112/http://www.ti.com/product/CC1101 "Ti CC1101"
[2]: https://github.com/dsvensson/cc1101 "cc1101 crate"
