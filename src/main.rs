#![no_main]
#![no_std]

use cortex_m_rt as rt;
use rt::entry;

use panic_rtt_core::{self, rprintln, rtt_init_print};

use bno080::wrapper::BNO080;
// use bno080::interface::{ I2cInterface, SpiInterface};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;

const IMU_REPORTING_RATE_HZ: u16 = 200;
const IMU_REPORTING_INTERVAL_MS: u16 = (1000 / IMU_REPORTING_RATE_HZ);

// type ImuDriverType = bno080::wrapper::BNO080<I2cInterface<ImuI2cPortType>>;
// type ImuDriverType = bno080::wrapper::BNO080<SpiInterface<SpiPortType, ChipSelectPinType, HIntPinType, WakePinType, ResetPinType>>;

#[cfg(feature = "nrf52832")]
mod peripherals_nrf52;
#[cfg(feature = "nrf52832")]
use peripherals_nrf52 as peripherals;

#[cfg(feature = "stm32f3x")]
mod peripherals_stm32f3x;
#[cfg(feature = "stm32f3x")]
use peripherals_stm32f3x as peripherals;

#[cfg(feature = "stm32f4x")]
mod peripherals_stm32f4x;
#[cfg(feature = "stm32f4x")]
use peripherals_stm32f4x as peripherals;

#[cfg(feature = "stm32h7x")]
mod peripherals_stm32h7x;
#[cfg(feature = "stm32h7x")]
use peripherals_stm32f4x as peripherals;


#[entry]
fn main() -> ! {
    rtt_init_print!(NoBlockTrim);
    rprintln!("-- > MAIN --");

    let (mut user_led1, mut delay_source, _i2c_port, mut _spi_control_lines) =
        peripherals::setup_peripherals();

    // SPI interface
    let iface = bno080::interface::SpiInterface::new(_spi_control_lines);

    // I2C interface
    // let iface = bno080::interface::I2cInterface::default(_i2c_port);

    let mut imu_driver = BNO080::new_with_interface(iface);
    imu_driver.init(&mut delay_source).unwrap();

    //cortex_m::asm::bkpt();
    imu_driver
        .enable_rotation_vector(IMU_REPORTING_INTERVAL_MS)
        .unwrap();

    let loop_interval = IMU_REPORTING_INTERVAL_MS as u8;
    rprintln!("loop_interval: {}", loop_interval);

    let _ = user_led1.set_low();

    loop {
        let _msg_count = imu_driver.handle_all_messages(&mut delay_source, 1u8);
        if _msg_count > 0 { rprintln!("> {}", _msg_count); }

        let _ = user_led1.toggle();
        delay_source.delay_ms(loop_interval);
    }
}
