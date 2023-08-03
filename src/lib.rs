pub mod i2c;
pub mod spi;

use std::error::Error as StdError;
use std::fmt;

// LSM6DSL
const LSM6DSL_ADDRESS: u16 = 0x6A;
const LSM6DSL_WHO_AM_I: u8 = 0x0F;
const LSM6DSL_CTRL1_XL: u8 = 0x10;
const LSM6DSL_CTRL8_XL: u8 = 0x17;
const LSM6DSL_CTRL2_G: u8 = 0x11;
const LSM6DSL_CTRL3_C: u8 = 0x12;
const LSM6DSL_OUTX_L_XL: u8 = 0x28;
const LSM6DSL_OUTX_H_XL: u8 = 0x29;
const LSM6DSL_OUTY_L_XL: u8 = 0x2A;
const LSM6DSL_OUTY_H_XL: u8 = 0x2B;
const LSM6DSL_OUTZ_L_XL: u8 = 0x2C;
const LSM6DSL_OUTZ_H_XL: u8 = 0x2D;
const LSM6DSL_OUTX_L_G: u8 = 0x22;
const LSM6DSL_OUTX_H_G: u8 = 0x23;
const LSM6DSL_OUTY_L_G: u8 = 0x24;
const LSM6DSL_OUTY_H_G: u8 = 0x25;
const LSM6DSL_OUTZ_L_G: u8 = 0x26;
const LSM6DSL_OUTZ_H_G: u8 = 0x27;

// LIS3MDL
const LIS3MDL_ADDRESS: u16 = 0x1C;
const LIS3MDL_WHO_AM_I: u8 = 0x0F;
const LIS3MDL_CTRL_REG1: u8 = 0x20;
const LIS3MDL_CTRL_REG2: u8 = 0x21;
const LIS3MDL_CTRL_REG3: u8 = 0x22;
const LIS3MDL_OUT_X_L: u8 = 0x28;

/// An error that occurred while interfacing with the BerryIMUv3 device.
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
