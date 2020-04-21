use stm32f3xx_hal as p_hal;

use p_hal::stm32 as pac;

use p_hal::stm32::I2C1;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use p_hal::flash::FlashExt;
use p_hal::gpio::GpioExt;
use p_hal::rcc::RccExt;
use p_hal::time::{Hertz, U32Ext};

use bno080::interface::dummy_output_pin::DummyOutputPin;
use bno080::interface::spi::SpiControlLines;

pub fn setup_peripherals() -> (
    impl OutputPin + ToggleableOutputPin,
    // GpioTypeUserLed1,
    impl DelayMs<u8>,
    ImuI2cPortType,
    BnoSpi1Lines,
) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let i2c_freq: Hertz = 400.khz().into();
    // Set up the system clock
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    // HSI: use default internal oscillator
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // HSE: external crystal oscillator must be connected
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz()) //72 works
        .pclk1(24.mhz()) // 24 works
        .freeze(&mut flash.acr);

    let delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    //stm32f334discovery:
    // let mut user_led1 = gpiob.pb6.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    // stm32f303 robodyn:
    let mut user_led1 = gpioc
        .pc13
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
    user_led1.set_high().unwrap();

    let i2c_port = {
        // setup i2c1 and imu driver
        let scl = gpiob
            .pb8
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
            .into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        let sda = gpiob
            .pb9
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
            .into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        p_hal::i2c::I2c::i2c1(
            dp.I2C1,
            (scl, sda),
            i2c_freq,
            clocks,
            &mut rcc.apb1,
        )
    };

    let spi_ctrl_lines = {
        // SPI1 port setup
        let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        // let sck = gpiob.pb3
        //     .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
        //     .into_af5(&mut gpiob.moder, &mut gpiob.afrl);
        // let miso = gpiob.pb4
        //     .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        //     .into_af5(&mut gpiob.moder, &mut gpiob.afrl);
        // let mosi = gpiob.pb5
        //     .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
        //     .into_af5(&mut gpiob.moder, &mut gpiob.afrl);

        let spi_port = p_hal::spi::Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            embedded_hal::spi::MODE_0,
            3_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );

        // SPI chip select CS
        let csn = gpioa
            .pa15
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        //.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        // HINTN interrupt pin
        let hintn = gpiob
            .pb0
            .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);

        // WAKEN pin / PS0
        let waken = DummyOutputPin::new();

        // let waken = gpiob
        //     .pb1
        //     .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        // .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        // NRSTN pin
        let reset_pin = gpiob
            .pb10
            //.into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

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

type ImuI2cPortType = p_hal::i2c::I2c<
    I2C1,
    (
        p_hal::gpio::gpiob::PB8<p_hal::gpio::AF4>,
        p_hal::gpio::gpiob::PB9<p_hal::gpio::AF4>,
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
type WakePinType = bno080::interface::dummy_output_pin::DummyOutputPin; // WAKE
                                                                        // p_hal::gpio::gpiob::PB1<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //PushPull>>; // WAKE
type ResetPinType =
    p_hal::gpio::gpiob::PB10<p_hal::gpio::Output<p_hal::gpio::PushPull>>; // RESET //OpenDrain

pub type BnoSpi1Lines = SpiControlLines<
    Spi1PortType,
    ChipSelectPinType,
    HIntPinType,
    WakePinType,
    ResetPinType,
>;
