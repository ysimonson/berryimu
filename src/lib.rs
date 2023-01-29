use std::error::Error as StdError;
use std::fmt;

use i2cdev::core::*;
#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

// LSM6DSL
pub const LSM6DSL_ADDRESS: u16 = 0x6A;
const LSM6DSL_WHO_AM_I: u8 = 0x0F;
const LSM6DSL_CTRL1_XL: u8 = 0x10;
const LSM6DSL_CTRL8_XL: u8 = 0x17;
const LSM6DSL_CTRL3_C: u8 = 0x12;
const LSM6DSL_OUTX_L_XL: u8 = 0x28;

// LIS3MDL
pub const LIS3MDL_ADDRESS: u16 = 0x1C;
const LIS3MDL_WHO_AM_I: u8 = 0x0F;
const LIS3MDL_CTRL_REG1: u8 = 0x20;
const LIS3MDL_CTRL_REG2: u8 = 0x21;
const LIS3MDL_CTRL_REG3: u8 = 0x22;
const LIS3MDL_OUT_X_L: u8 = 0x28;

#[derive(Debug)]
pub enum Error<E: StdError + 'static> {
    Init,
    Read,
    Write,
    Device(E),
}

impl<E: StdError + 'static> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Device(ref err) => Some(err),
            _ => None,
        }
    }
}

impl<E: StdError + 'static> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Init => write!(f, "init failed"),
            Error::Read => write!(f, "read failed"),
            Error::Write => write!(f, "write failed"),
            Error::Device(err) => write!(f, "device error: {}", err),
        }
    }
}

impl<E: StdError + 'static> From<E> for Error<E> {
    fn from(err: E) -> Self {
        Error::Device(err)
    }
}

fn init<D: I2CDevice>(
    dev: &mut D,
    who_am_i: u8,
    expected_response: u8,
) -> Result<(), Error<D::Error>> {
    let lsm6dsl_who_m_response = dev.smbus_read_byte_data(who_am_i)?;
    if lsm6dsl_who_m_response == expected_response {
        Ok(())
    } else {
        Err(Error::Init)
    }
}

fn read_block<D: I2CDevice>(
    dev: &mut D,
    command: u8,
    size: u8,
) -> Result<Vec<u8>, Error<D::Error>> {
    let block = dev.smbus_read_i2c_block_data(command, size)?;
    if block.len() != size as usize {
        return Err(Error::Read);
    }
    Ok(block)
}

pub struct Accelerometer<D: I2CDevice>(D);

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Accelerometer<LinuxI2CDevice> {
    pub fn new_on_linux<P: AsRef<Path>>(addr: P) -> Result<Self, Error<D::Error>> {
        let mut dev = LinuxI2CDevice::new(addr, LSM6DSL_ADDRESS)?;
        Accelerometer::new(dev)
    }
}

impl<D: I2CDevice> Accelerometer<D> {
    pub fn new(mut dev: D) -> Result<Self, Error<D::Error>> {
        init(&mut dev, LSM6DSL_WHO_AM_I, 0x6A)?;
        // Enable the accelerometer
        dev.smbus_write_byte_data(LSM6DSL_CTRL1_XL, 0b10011111)?; // ODR 3.33 kHz, +/- 8g , BW = 400hz
        dev.smbus_write_byte_data(LSM6DSL_CTRL8_XL, 0b11001000)?; // Low pass filter enabled, BW9, composite filter
        dev.smbus_write_byte_data(LSM6DSL_CTRL3_C, 0b01000100)?; // Enable Block Data update, increment during multi byte read
        Ok(Self(dev))
    }

    pub fn read(&mut self) -> Result<(i32, i32, i32), Error<D::Error>> {
        let block = read_block(&mut self.0, LSM6DSL_OUTX_L_XL, 6)?;
        // Combine readings for each axis
        let x = ((block[0] as i16) | (block[1] as i16) << 8) as i32;
        let y = ((block[2] as i16) | (block[3] as i16) << 8) as i32;
        let z = ((block[4] as i16) | (block[5] as i16) << 8) as i32;
        Ok((x, y, z))
    }
}

pub struct Magnetometer<D: I2CDevice>(D);

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Magnetometer<LinuxI2CDevice> {
    pub fn new_on_linux<P: AsRef<Path>>(addr: P) -> Result<Self, Error<D::Error>> {
        let mut dev = LinuxI2CDevice::new(addr, LIS3MDL_ADDRESS)?;
        Magnetometer::new(dev)
    }
}

impl<D: I2CDevice> Magnetometer<D> {
    pub fn new(mut dev: D) -> Result<Self, Error<D::Error>> {
        init(&mut dev, LIS3MDL_WHO_AM_I, 0x3D)?;
        // Enable the magnetometer
        dev.smbus_write_byte_data(LIS3MDL_CTRL_REG1, 0b11011100)?; // Temp sensor enabled, High performance, ODR 80 Hz, FAST ODR disabled and Selft test disabled.
        dev.smbus_write_byte_data(LIS3MDL_CTRL_REG2, 0b00100000)?; // +/- 8 gauss
        dev.smbus_write_byte_data(LIS3MDL_CTRL_REG3, 0b00000000)?; // Continuous-conversion mode
        Ok(Self(dev))
    }

    pub fn read(&mut self) -> Result<(i32, i32, i32), Error<D::Error>> {
        let block = read_block(&mut self.0, LIS3MDL_OUT_X_L, 6)?;
        // Combine readings for each axis
        let x = ((block[0] as i16) | (block[1] as i16) << 8) as i32;
        let y = ((block[2] as i16) | (block[3] as i16) << 8) as i32;
        let z = ((block[4] as i16) | (block[5] as i16) << 8) as i32;
        Ok((x, y, z))
    }
}
