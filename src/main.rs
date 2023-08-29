use crossbeam_channel::bounded;
use log::*;
use std::{
    io::{self, BufRead, BufReader},
    thread::{self, sleep},
    time::Duration,
};

use embedded_ccs811::{nb::block, prelude::*, Ccs811Awake};
use esp_idf_hal::{i2c::*, prelude::*, peripherals::Peripherals};
use esp_idf_sys as _;
use scd30::Scd30;

mod blocking_reader;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Setup I2C
    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio6;
    let scl = peripherals.pins.gpio7;
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();
    let i2c: &'static _ = shared_bus::new_std!(I2cDriver = i2c).unwrap();

    // Setup scd-30 sensor
    let mut scd30 = Scd30::new(i2c.acquire_i2c());
    // Setup ccs-811 sensor
    let mut ccs811 = loop {
        let ccs811 = Ccs811Awake::new(i2c.acquire_i2c(), embedded_ccs811::SlaveAddr::default());
        match ccs811.start_application() {
            Ok(mut new_ccs811) => {
                match new_ccs811.set_mode(embedded_ccs811::MeasurementMode::ConstantPower1s) {
                    Ok(_) => break new_ccs811,
                    Err(error) => {
                        error!("{:?}", error);
                        continue;
                    }
                }
            }
            Err(embedded_ccs811::ModeChangeError { dev: _, error }) => {
                error!("{:?}", error);
                continue;
            }
        }
    };

    let (request_tx, request_rx) = bounded(5);
    let (reply_tx, reply_rx) = bounded(5);

    let local_reply_tx = reply_tx.clone();
    thread::spawn(move || loop {
        request_rx.recv().unwrap();
        local_reply_tx
            .send((scd30.read(), block!(ccs811.data())))
            .unwrap();
    });

    let local_reply_rx = reply_rx.clone();
    let local_request_tx = request_tx.clone();
    thread::spawn(move || loop {
        local_request_tx.send(()).unwrap();

        let (scd30_data, ccs811_data) = local_reply_rx.recv().unwrap();
        handle_scd30_measurement(scd30_data);
        handle_ccs811_measurement(ccs811_data);

        sleep(Duration::from_secs(60 * 10));
    });

    let stdin: blocking_reader::BlockingReader<_> = io::stdin().into();
    let mut stdin = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let _ = stdin.read_line(&mut line).unwrap();

        request_tx.send(()).unwrap();
        println!("{:#?}", reply_rx.recv().unwrap());
    }
}

fn handle_scd30_measurement(reading: Result<Option<scd30::Measurement>, I2cError>) {
    match reading {
        Ok(reading) => match reading {
            Some(measurement) => {
                info!("scd30: {:?}", measurement);
            }
            None => {
                info!("scd30: No data is available from the sensor");
            }
        },
        Err(err) => {
            info!("scd30: Sensor error: {:?}", err);
        }
    }
}

fn handle_ccs811_measurement(
    reading: Result<embedded_ccs811::AlgorithmResult, embedded_ccs811::ErrorAwake<I2cError>>,
) {
    match reading {
        Ok(data) => info!("ccs811: {:?}", data),
        Err(err) => info!("ccs811: Sensor error: {:?}", err),
    }
}
