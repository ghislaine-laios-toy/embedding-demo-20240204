use std::cell::RefCell;
use anyhow::Result;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::i2c;
use esp_idf_svc::hal::{
    delay::FreeRtos, i2c::{I2cConfig, I2cDriver}, peripherals::Peripherals, prelude::*,
};
use esp_idf_svc::hal::i2c::I2cError;
use mlx9061x::{Mlx9061x, SlaveAddr};
use mlx9061x::ic::Mlx90614;
use thiserror::Error;


#[allow(unreachable_code)]
fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world! APP has started.");

    let peripherals = Peripherals::take()?;

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let baudrate = 100u32.kHz();
    let config = I2cConfig::new().baudrate(baudrate.into());

    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;
    let i2c = RefCell::new(i2c);
    let mxl90614_i2c = i2c::RefCellDevice::new(&i2c);
    let bno055_i2c = i2c::RefCellDevice::new(&i2c);

    // Setup the mxl90614 temperature sensor.

    let mxl90614_address = SlaveAddr::Alternative(0x5a);
    let mxl90614_device = Mlx9061x::new_mlx90614(mxl90614_i2c, mxl90614_address, 5);
    let mut mxl90614_device = mxl90614_device.or_else(|e| Err(Mxl9061xError::Generic(e)))?;

    let id = mxl90614_device.device_id().or_else(|e| Err(Mxl9061xError::GetDeviceID(e)))?;
    log::info!("Connected to MXL90614 sensor. ID: {}.", id);

    // Setup bno055 IMU.

    loop {
        FreeRtos.delay_ms(500);

        // Splitting into two steps for a bug of RustRover.
        let obj1_temp = mxl90614_device.object1_temperature()
            .map_err(|e| Mxl9061xError::GetObject1Temperature(e));
        let obj1_temp = obj1_temp?;

        let obj2_temp = mxl90614_device.object2_temperature()
            .map_err(|e| Mxl9061xError::GetObject2Temperature(e));
        let obj2_temp = obj2_temp?;

        let ambient_temp = mxl90614_device.ambient_temperature()
            .map_err(|e| Mxl9061xError::GetAmbientTemperature(e));
        let ambient_temp = ambient_temp?;

        log::info!("Temp: obj1 - {}, obj2 - {}, ambient - {}", obj1_temp, obj2_temp, ambient_temp);
    }

    return Ok(());
}

#[derive(Error, Debug)]
enum Mxl9061xError {
    #[error("An error is emitted by MLX9061X driver.")]
    Generic(mlx9061x::Error<I2cError>),
    #[error("When getting the device ID of the MLX9061 device, an error occurred.")]
    GetDeviceID(mlx9061x::Error<I2cError>),
    #[error("When getting the temperature of object1 from the MLX9061 device, an error occurred.")]
    GetObject1Temperature(mlx9061x::Error<I2cError>),
    #[error("When getting the temperature of object2 from the MLX9061 device, an error occurred.")]
    GetObject2Temperature(mlx9061x::Error<I2cError>),
    #[error("When getting the ambient temperature from the MLX9061 device, an error occurred.")]
    GetAmbientTemperature(mlx9061x::Error<I2cError>),
}

struct Mxl90614DeviceDemo<I2C> {
    device: Mlx9061x<I2C, Mlx90614>
}