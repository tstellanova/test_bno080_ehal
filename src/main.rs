
#![no_main]
#![no_std]

use cortex_m_rt as rt;
use rt::{entry};

use panic_semihosting as _;

use bno080::wrapper::BNO080;
// use bno080::interface::{ I2cInterface, SpiInterface};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_hal::blocking::delay::DelayMs;


const IMU_REPORTING_RATE_HZ: u16 = 100;
const IMU_REPORTING_INTERVAL_MS: u16 = (1000 / IMU_REPORTING_RATE_HZ) ;


// type ImuDriverType = bno080::wrapper::BNO080<I2cInterface<ImuI2cPortType>>;
// type ImuDriverType = bno080::wrapper::BNO080<SpiInterface<SpiPortType, ChipSelectPinType, HIntPinType, WakePinType, ResetPinType>>;


#[cfg(not(feature = "nrf52832"))]
mod peripherals_stm32;
#[cfg(not(feature = "nrf52832"))]
use peripherals_stm32 as peripherals;

#[cfg(feature = "nrf52832")]
mod peripherals_nrf52;
#[cfg(feature = "nrf52832")]
use peripherals_nrf52 as peripherals;



#[entry]
fn main() -> ! {

    let (mut user_led1, mut delay_source, i2c_port, spi_control_lines) =
        peripherals::setup_peripherals();

    let spi_iface = bno080::interface::SpiInterface::new(spi_control_lines);
    let _i2c_iface = bno080::interface::I2cInterface::new(i2c_port, bno080::interface::i2c::DEFAULT_ADDRESS);

    let mut imu_driver = BNO080::new_with_interface(spi_iface);
    imu_driver.init(&mut delay_source).unwrap();
    imu_driver.enable_rotation_vector(IMU_REPORTING_INTERVAL_MS).unwrap();

    let _ = user_led1.set_low();

    loop {
        let msg_count  = imu_driver.handle_all_messages(&mut delay_source);
        //hprintln!("> {}", msg_count).unwrap();
        let _ = user_led1.toggle();
        delay_source.delay_ms(25u8);
    }

}


