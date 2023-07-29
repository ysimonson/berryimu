use i2cdev::core::*;
#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::path::Path;

fn init<D: I2CDevice>(
    dev: &mut D,
    who_am_i: u8,
    expected_response: u8,
) -> Result<(), crate::Error<D::Error>> {
    let who_am_i_response = dev.smbus_read_byte_data(who_am_i)?;
    if who_am_i_response == expected_response {
        Ok(())
    } else {
        Err(crate::Error::Init)
    }
}

fn read_block<D: I2CDevice>(
    dev: &mut D,
    command: u8,
    size: u8,
) -> Result<Vec<u8>, crate::Error<D::Error>> {
    let block = dev.smbus_read_i2c_block_data(command, size)?;
    if block.len() != size as usize {
        return Err(crate::Error::Read);
    }
    Ok(block)
}

/// An accelerometer reader.
pub struct Accelerometer<D: I2CDevice>(D);

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Accelerometer<LinuxI2CDevice> {
    /// Creates a new accelerometer reader from an address.
    ///
    /// # Arguments
    /// * `addr`: The I2C device address, e.g. `/dev/i2c-1`.
    pub fn new_from_address<P: AsRef<Path>>(addr: P) -> Result<Self, crate::Error<LinuxI2CError>> {
        let dev = LinuxI2CDevice::new(addr, crate::LSM6DSL_ADDRESS)?;
        Accelerometer::new(dev)
    }
}

impl<D: I2CDevice> Accelerometer<D> {
    /// Creates a new accelerometer reader from an I2C device.
    ///
    /// # Arguments
    /// * `dev`: The I2C device.
    pub fn new(mut dev: D) -> Result<Self, crate::Error<D::Error>> {
        init(&mut dev, crate::LSM6DSL_WHO_AM_I, 0x6A)?;
        dev.smbus_write_byte_data(crate::LSM6DSL_CTRL1_XL, 0b10011111)?; // ODR 3.33 kHz, +/- 8g , BW = 400hz
        dev.smbus_write_byte_data(crate::LSM6DSL_CTRL8_XL, 0b11001000)?; // Low pass filter enabled, BW9, composite filter
        dev.smbus_write_byte_data(crate::LSM6DSL_CTRL3_C, 0b01000100)?; // Enable Block Data update, increment during multi byte read
        Ok(Self(dev))
    }

    /// Read the raw accelerometer values.
    pub fn read(&mut self) -> Result<(i32, i32, i32), crate::Error<D::Error>> {
        let block = read_block(&mut self.0, crate::LSM6DSL_OUTX_L_XL, 6)?;
        // Combine readings for each axis
        let x = ((block[0] as i16) | (block[1] as i16) << 8) as i32;
        let y = ((block[2] as i16) | (block[3] as i16) << 8) as i32;
        let z = ((block[4] as i16) | (block[5] as i16) << 8) as i32;
        Ok((x, y, z))
    }
}

/// A magnetometer reader.
pub struct Magnetometer<D: I2CDevice>(D);

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Magnetometer<LinuxI2CDevice> {
    /// Creates a new magnetometer reader from an address.
    ///
    /// # Arguments
    /// * `addr`: The I2C device address, e.g. `/dev/i2c-1`.
    pub fn new_from_address<P: AsRef<Path>>(addr: P) -> Result<Self, crate::Error<LinuxI2CError>> {
        let dev = LinuxI2CDevice::new(addr, crate::LIS3MDL_ADDRESS)?;
        Magnetometer::new(dev)
    }
}

impl<D: I2CDevice> Magnetometer<D> {
    /// Creates a new magnetometer reader from an I2C device.
    ///
    /// # Arguments
    /// * `dev`: The I2C device.
    pub fn new(mut dev: D) -> Result<Self, crate::Error<D::Error>> {
        init(&mut dev, crate::LIS3MDL_WHO_AM_I, 0x3D)?;
        // Enable the magnetometer
        dev.smbus_write_byte_data(crate::LIS3MDL_CTRL_REG1, 0b11011100)?; // Temp sensor enabled, High performance, ODR 80 Hz, FAST ODR disabled and Selft test disabled.
        dev.smbus_write_byte_data(crate::LIS3MDL_CTRL_REG2, 0b00100000)?; // +/- 8 gauss
        dev.smbus_write_byte_data(crate::LIS3MDL_CTRL_REG3, 0b00000000)?; // Continuous-conversion mode
        Ok(Self(dev))
    }

    /// Read the raw magnetometer values.
    pub fn read(&mut self) -> Result<(i32, i32, i32), crate::Error<D::Error>> {
        let block = read_block(&mut self.0, crate::LIS3MDL_OUT_X_L, 6)?;
        // Combine readings for each axis
        let x = ((block[0] as i16) | (block[1] as i16) << 8) as i32;
        let y = ((block[2] as i16) | (block[3] as i16) << 8) as i32;
        let z = ((block[4] as i16) | (block[5] as i16) << 8) as i32;
        Ok((x, y, z))
    }
}
