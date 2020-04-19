


use nrf52832_hal as p_hal;
use p_hal::nrf52832_pac as pac;
use p_hal::{
    clocks::ClocksExt,
    gpio::{GpioExt, Level},
};

use p_hal::{delay::Delay, spim, twim};

use p_hal::time::{U32Ext, Hertz};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use bno080::interface::spi::SpiControlLines;

pub type Spi1PortType = p_hal::spim::Spim<pac::SPIM0>;
// p_hal::spi::Spi<pac::SPI1,
//     (
//         p_hal::gpio::gpioa::PA5<p_hal::gpio::AF5>, //SCLK
//         p_hal::gpio::gpioa::PA6<p_hal::gpio::AF5>, //MISO?
//         p_hal::gpio::gpioa::PA7<p_hal::gpio::AF5>, //MOSI?
//     )
// >;

type ChipSelectPinType = p_hal::gpio::p0::P0_25<p_hal::gpio::Output<p_hal::gpio::PushPull>>;   //CSN
type HIntPinType =   p_hal::gpio::Pin<p_hal::gpio::Input<p_hal::gpio::Floating>>; //HINTN
type WakePinType =  p_hal::gpio::p0::P0_18<p_hal::gpio::Output<p_hal::gpio::PushPull>>; // WAKE
type ResetPinType =  p_hal::gpio::p0::P0_26<p_hal::gpio::Output<p_hal::gpio::PushPull>>;  // RESET

pub type BnoSpi1Lines = SpiControlLines<Spi1PortType, ChipSelectPinType, HIntPinType, WakePinType, ResetPinType>;


pub fn setup_peripherals() -> (
    impl OutputPin + ToggleableOutputPin,
    impl  DelayMs<u8>,
    ImuI2cPortType,
    BnoSpi1Lines,
)
{
    let cp = pac::CorePeripherals::take().unwrap();
    let mut delay_source = Delay::new(cp.SYST);

    // PineTime has a 32 MHz HSE (HFXO) and a 32.768 kHz LSE (LFXO)
    let mut dp = pac::Peripherals::take().unwrap();
    let _clockit = dp
        .CLOCK
        .constrain()
        //.set_lfclk_src_external(LfOscConfiguration::ExternalAndBypass)
        .start_lfclk()
        .enable_ext_hfosc();
    // TODO configure low-speed clock with LfOscConfiguration: currently hangs
    //.set_lfclk_src_external(LfOscConfiguration::ExternalNoBypass).start_lfclk();

    let port0 = dp.P0.split();

    // random number generator peripheral
    //let mut rng = dp.RNG.constrain();
    let mut user_led1 =
        port0.p0_17.into_push_pull_output(Level::High);

    let mut _user_butt = port0.p0_13.into_floating_input().degrade();

    let i2c0_pins = twim::Pins {
        scl: port0.p0_07.into_floating_input().degrade(),
        sda: port0.p0_06.into_floating_input().degrade(),
    };
    let i2c_port = twim::Twim::new(dp.TWIM1, i2c0_pins, twim::Frequency::K400);


    let spi_ctrl_lines = {
        let spim0_pins = spim::Pins {
            sck: port0.p0_02.into_push_pull_output(Level::Low).degrade(),
            miso: None,
            mosi: Some(port0.p0_03.into_push_pull_output(Level::Low).degrade()),
        };

        // create SPIM0 interface, 8 Mbps, use 122 as "over read character"
        let spim0 = spim::Spim::new(
            dp.SPIM0,
            spim0_pins,
            spim::Frequency::M8,
            spim::MODE_3,
            122,
        );

        // SPI chip select CS
        let csn =
            port0.p0_25.into_push_pull_output(Level::High);

        // HINTN interrupt pin
        let hintn =
            port0.p0_13.into_floating_input().degrade();

        // WAKEN pin / PS0
        let waken =
            port0.p0_18.into_push_pull_output(Level::High);

        // NRSTN pin
        let reset_pin =
            port0.p0_26.into_push_pull_output(Level::High);

        // let flash_csn = port0.p0_05.into_push_pull_output(Level::High);

        SpiControlLines {
            spi: spim0,
            csn,
            hintn,
            waken,
            reset: reset_pin,
        }
    };

    ( user_led1, delay_source, i2c_port, spi_ctrl_lines)

}


type ImuI2cPortType = p_hal::twim::Twim<pac::TWIM1>;
// type ImuI2cPortType = p_hal::i2c::I2c<I2C1,
//     (p_hal::gpio::gpiob::PB8<p_hal::gpio::AF4>,
//      p_hal::gpio::gpiob::PB9<p_hal::gpio::AF4>)
// >;