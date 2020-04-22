use stm32f4xx_hal as p_hal;

use p_hal::stm32 as pac;
use p_hal::stm32::I2C1;

// use p_hal::flash::FlashExt;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use p_hal::gpio::GpioExt;
use p_hal::rcc::RccExt;
use p_hal::time::{U32Ext};

use bno080::interface::spi::SpiControlLines;
use bno080::interface::dummy_output_pin::DummyOutputPin;

pub fn setup_peripherals() -> (
    impl OutputPin + ToggleableOutputPin,
    impl DelayMs<u8>,
    ImuI2cPortType,
    BnoSpi1Lines,
) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Set up the system clock
    let rcc = dp.RCC.constrain();
    // HSI: use default internal oscillator
    //let clocks = rcc.cfgr.freeze();
    // HSE: external crystal oscillator must be connected
    // let clocks = rcc
    //     .cfgr
    //     .use_hse(8.mhz()) //f4 discovery board has 8 MHz crystal for HSE
    //     .sysclk(128.mhz())
    //     .pclk1(48.mhz())
    //     // .pclk2(48.mhz())
    //     .freeze();

    let clocks = rcc
        .cfgr
        .use_hse(25.mhz()) //f401cb  board has 25 MHz crystal for HSE
        .sysclk(72.mhz())
        .pclk1(48.mhz())
        .pclk2(48.mhz()) // required for spi1
        .freeze();

    let delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);


    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let user_led1 = gpioc.pc13.into_push_pull_output(); //f401CxUx
                                                        // let user_led1 = gpiod.pd12.into_push_pull_output(); //f4discovery

    // setup i2c1
    // NOTE: stm32f401CxUx board lacks external pull-ups on i2c pins
    // NOTE: eg f407 discovery board already has external pull-ups
    // NOTE: sensor breakout boards may have their own pull-ups: check carefully
    let i2c_port = {
        let scl = gpiob
            .pb8
            .into_alternate_af4()
            //.internal_pull_up(true)
            .set_open_drain();

        let sda = gpiob
            .pb9
            .into_alternate_af4()
            //.internal_pull_up(true)
            .set_open_drain();

        p_hal::i2c::I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks)
    };

    let spi_ctrl_lines = {
        // SPI1 port setup
        let sck = gpioa.pa5.into_alternate_af5();
        let miso = gpioa.pa6.into_alternate_af5();
        let mosi = gpioa.pa7.into_alternate_af5();

        let spi_port = p_hal::spi::Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            embedded_hal::spi::MODE_3,
            3_000_000.hz(),
            clocks,
        );

        // SPI chip select CS
        let mut csn = gpiob.pb0.into_push_pull_output();
        let _ = csn.set_high();

        // HINTN interrupt pin
        let hintn = gpiob.pb1.into_pull_up_input();

        // WAKEN pin / PS0
        let waken = DummyOutputPin::new();

        // NRSTN pin
        let mut reset_pin = gpiob.pb10.into_push_pull_output();
        let _ = reset_pin.set_high();

        SpiControlLines {
            spi: spi_port,
            csn,
            hintn,
            waken,
            reset: reset_pin,
        }
    };

    (user_led1, delay_source, i2c_port, spi_ctrl_lines)
}

pub type ImuI2cPortType = p_hal::i2c::I2c<
    I2C1,
    (
        p_hal::gpio::gpiob::PB8<p_hal::gpio::AlternateOD<p_hal::gpio::AF4>>,
        p_hal::gpio::gpiob::PB9<p_hal::gpio::AlternateOD<p_hal::gpio::AF4>>,
    ),
>;

pub type Spi1PortType = p_hal::spi::Spi<
    pac::SPI1,
    (
        p_hal::gpio::gpioa::PA5<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //SCLK
        p_hal::gpio::gpioa::PA6<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //MISO
        p_hal::gpio::gpioa::PA7<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //MOSI
    ),
>;

type ChipSelectPinType =
    p_hal::gpio::gpiob::PB0<p_hal::gpio::Output<p_hal::gpio::PushPull>>; //CSN

type HIntPinType =
    p_hal::gpio::gpiob::PB1<p_hal::gpio::Input<p_hal::gpio::PullUp>>; //HINTN

type WakePinType = bno080::interface::dummy_output_pin::DummyOutputPin; // WAKE

type ResetPinType =
    p_hal::gpio::gpiob::PB10<p_hal::gpio::Output<p_hal::gpio::PushPull>>; // RESET //OpenDrain

pub type BnoSpi1Lines = SpiControlLines<
    Spi1PortType,
    ChipSelectPinType,
    HIntPinType,
    WakePinType,
    ResetPinType,
>;
