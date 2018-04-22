## IKEA Sparsnäs receiver

Work in progress, should be configurable, and perhaps turned into a library and another
application would do the actual work of storing the data in for example Influx DB. Currently
CRC check doesn't pass in the cc1101 crate, should be fixed too. I'm fairly new with Rust,
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

## License

Licensed under your option of:

* [MIT License](LICENSE-MIT)
* [Apache License, Version 2.0](LICENSE-APACHE)
