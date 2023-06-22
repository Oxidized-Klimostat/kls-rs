use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use scd30::scd30::Scd30;
use std::{thread, time};


fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio7;
    let scl = peripherals.pins.gpio6;


    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    let mut scd30 = Scd30::new(i2c);

    loop {
        if scd30.data_ready().unwrap() {
            break;
        }
        thread::sleep(time::Duration::from_secs(1));
    }

    loop {
        println!("{:?}", scd30.read().unwrap().unwrap());
        thread::sleep(time::Duration::from_secs(20));
    }
}
