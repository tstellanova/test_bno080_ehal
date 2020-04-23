# test_bno080_ehal

Utilities for testing the bno080 device driver on multiple MCU families


# Running

`cargo run` should just work if you have everything connected properly. 

Note that by default this example assumes you've got a Segger J-Link connected
to your MCU target.  If you do, `cargo run` will install and run the test application.
If you also have eg the JLinkRTTViewer open, you should see output. 

### Selecting an MCU 
To select a particular MCU, use the default line in [Cargo.toml](./Cargo.toml):
```
default = ["stm32f3x", "panic-rtt-core"]
```

Set the first item to the MCU family you're trying to use.
Note that we've only tested with a limited set of MCUs, as detailed in Cargo.toml.
If you're not using exactly the MCU and board we tested with, you'll need to carefully
select your own pins.

### Switching between SPI and I2C interfaces

In [main.rs](./src/main.rs) you can switch between using I2C and SPI for the interface:

```rust
    // SPI interface
    let iface = bno080::interface::SpiInterface::new(_spi_control_lines);

    // I2C interface
    // let iface = bno080::interface::I2cInterface::default(_i2c_port);
```