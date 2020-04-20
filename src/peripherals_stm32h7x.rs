use stm32h7xx_hal as p_hal;

use p_hal::stm32 as pac;

use p_hal::stm32::I2C1;

// use p_hal::flash::FlashExt;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use p_hal::gpio::GpioExt;
use p_hal::rcc::RccExt;
use p_hal::time::{Hertz, U32Ext};

use bno080::interface::spi::SpiControlLines;

pub fn setup_peripherals() -> (
    impl OutputPin + ToggleableOutputPin,
    impl DelayMs<u8>,
    ImuI2cPortType,
    BnoSpi1Lines,
) {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Set up the system clock
    let rcc = dp.RCC.constrain();

    let pwr = dp.PWR.constrain();
    let vos = pwr.freeze();

    //use the existing sysclk
    let mut ccdr = rcc.freeze(vos, &dp.SYSCFG);
    let clocks = ccdr.clocks;

    let delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);

    let gpiob = dp.GPIOB.split(&mut ccdr.ahb4);

    let user_led1 = gpiob.pb0.into_push_pull_output(); //h743 discovery

    // TODO setup i2c1
    // NOTE:  h743 discovery board already has external pull-ups?
    let scl = gpiob
        .pb8
        .into_alternate_af4()
        // .internal_pull_up(true)
        .set_open_drain();

    let sda = gpiob
        .pb9
        .into_alternate_af4()
        // .internal_pull_up(true)
        .set_open_drain();
    let i2c_port = p_hal::i2c::I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), &ccdr);

    (i2c_port, user_led1, delay_source)
}

pub type ImuI2cPortType = p_hal::i2c::I2c<
    I2C1,
    (
        p_hal::gpio::gpiob::PB8<p_hal::gpio::Alternate<p_hal::gpio::AF4>>,
        p_hal::gpio::gpiob::PB9<p_hal::gpio::Alternate<p_hal::gpio::AF4>>,
    ),
>;

pub type Spi1PortType = p_hal::spi::Spi<
    pac::SPI1,
    (
        // p_hal::gpio::gpiob::PB3<p_hal::gpio::AF5>, //SCLK
        // p_hal::gpio::gpiob::PB4<p_hal::gpio::AF5>, //MISO?
        // p_hal::gpio::gpiob::PB5<p_hal::gpio::AF5>, //MOSI?
        p_hal::gpio::gpioa::PA5<p_hal::gpio::AF5>, //SCLK
        p_hal::gpio::gpioa::PA6<p_hal::gpio::AF5>, //MISO?
        p_hal::gpio::gpioa::PA7<p_hal::gpio::AF5>, //MOSI?
    ),
>;

type ChipSelectPinType =
    p_hal::gpio::gpioa::PA15<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //CSN
type HIntPinType =
    p_hal::gpio::gpiob::PB0<p_hal::gpio::Input<p_hal::gpio::PullUp>>; //HINTN
type WakePinType =
    p_hal::gpio::gpiob::PB1<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //PushPull>>; // WAKE
type ResetPinType =
    p_hal::gpio::gpiob::PB10<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; // RESET

pub type BnoSpi1Lines = SpiControlLines<
    Spi1PortType,
    ChipSelectPinType,
    HIntPinType,
    WakePinType,
    ResetPinType,
>;
