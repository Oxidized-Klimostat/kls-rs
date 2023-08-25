use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use log::*;
use scd30::Scd30;
use crossbeam_channel::unbounded;
use std::{
    io::{self, BufRead, BufReader},
    thread::{self, sleep},
    time::Duration,
};

mod blocking_reader;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio6;
    let scl = peripherals.pins.gpio7;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    let mut scd30 = Scd30::new(i2c);

    let (request_tx, request_rx) = unbounded();
    let (reply_tx, reply_rx) = unbounded();

    let local_reply_tx = reply_tx.clone();
    let data_thread = thread::spawn(move || {
        loop {
            request_rx.recv().unwrap();
            local_reply_tx.send(scd30.read()).unwrap();
        }
    });

    let local_reply_rx = reply_rx.clone();
    let local_request_tx = request_tx.clone();
    let info_thread = thread::spawn(move || {
        loop {
            local_request_tx.send(()).unwrap();

            match local_reply_rx.recv().unwrap() {
                Ok(reading) => match reading {
                    Some(measurement) => {
                        info!("Automatic measurement: {:?}", measurement);
                        sleep(Duration::from_secs(60 * 10)).await;
                    }
                    None => {
                        info!("Automatic measurement: No data is available from the sensor, waiting 5 seconds");
                        sleep(Duration::from_secs(5));
                    }
                },
                Err(_) => {
                    info!("Automatic measurement: Sensor not ready, waiting 5 seconds");
                    sleep(Duration::from_secs(5));
                }
            }
        }
    });

    let stdin: blocking_reader::BlockingReader<_> = io::stdin().into();
    let mut stdin = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let _ = stdin.read_line(&mut line).unwrap();

        request_tx.send(());
        println!("{:#?}", reply_rx.recv().unwrap());
    }
}
