use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::io;
use std::path::Path;

fn device_from_address<P: AsRef<Path>>(addr: P) -> io::Result<Spidev> {
    let mut dev = Spidev::open(addr)?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    dev.configure(&options)?;
    Ok(dev)
}

fn read_reg(dev: &mut Spidev, reg_address: u8) -> io::Result<u8> {
    // "write" transfers are also reads at the same time with the read having
    // the same length as the write.
    let tx_buf = [reg_address | 0x80, 0x00];
    let mut rx_buf = [0; 2];
    {
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        dev.transfer(&mut transfer)?;
    }
    Ok(rx_buf[1])
}

fn write_reg(dev: &mut Spidev, reg_address: u8, data: u8) -> io::Result<[u8; 2]> {
    // "write" transfers are also reads at the same time with the read having
    // the same length as the write.
    let tx_buf = [reg_address, data];
    let mut rx_buf = [0; 2];
    {
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        dev.transfer(&mut transfer)?;
    }
    Ok(rx_buf)
}

fn read_axis(dev: &mut Spidev, l_reg_address: u8, h_reg_address: u8) -> io::Result<i32> {
    let acc_l = read_reg(dev, l_reg_address)?;
    let acc_h = read_reg(dev, h_reg_address)?;
    let acc_combined: u16 = (acc_l as u16) | ((acc_h as u16) << 8);
    if acc_combined < 32768 {
        Ok(acc_combined as i32)
    } else {
        Ok((acc_combined as i32) - 65536)
    }
}

fn init(
    dev: &mut Spidev,
    who_am_i: u8,
    expected_response: u8,
) -> Result<(), crate::Error<io::Error>> {
    let who_am_i_response = read_reg(dev, who_am_i)?;
    if who_am_i_response == expected_response {
        Ok(())
    } else {
        Err(crate::Error::Init)
    }
}

/// An accelerometer reader.
pub struct Accelerometer(Spidev);

impl Accelerometer {
    /// Creates a new accelerometer reader from an address.
    ///
    /// # Arguments
    /// * `addr`: The SPI device address, e.g. `/dev/spidev0.0`.
    pub fn new_from_address<P: AsRef<Path>>(addr: P) -> Result<Self, crate::Error<io::Error>> {
        Accelerometer::new(device_from_address(addr)?)
    }

    /// Creates a new accelerometer reader from a SPI device.
    ///
    /// # Arguments
    /// * `dev`: The SPI device.
    pub fn new(mut dev: Spidev) -> Result<Self, crate::Error<io::Error>> {
        init(&mut dev, crate::LSM6DSL_WHO_AM_I, 0x6A)?;
        write_reg(&mut dev, crate::LSM6DSL_CTRL1_XL, 0b10011111)?; // ODR 3.33 kHz, +/- 8g , BW = 400hz
        write_reg(&mut dev, crate::LSM6DSL_CTRL8_XL, 0b11001000)?; // Low pass filter enabled, BW9, composite filter
        write_reg(&mut dev, crate::LSM6DSL_CTRL3_C, 0b01000100)?; // Enable Block Data update, increment during multi byte read
        Ok(Self(dev))
    }

    /// Read the raw accelerometer values.
    pub fn read(&mut self) -> Result<(i32, i32, i32), crate::Error<io::Error>> {
        let x = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTX_L_XL,
            crate::LSM6DSL_OUTX_H_XL,
        )?;
        let y = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTY_L_XL,
            crate::LSM6DSL_OUTY_H_XL,
        )?;
        let z = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTZ_L_XL,
            crate::LSM6DSL_OUTZ_H_XL,
        )?;
        Ok((x, y, z))
    }
}

/// A gyroscope reader.
pub struct Gyroscope(Spidev);

impl Gyroscope {
    /// Creates a new gyroscope reader from an address.
    ///
    /// # Arguments
    /// * `addr`: The SPI device address, e.g. `/dev/spidev0.0`.
    pub fn new_from_address<P: AsRef<Path>>(addr: P) -> Result<Self, crate::Error<io::Error>> {
        Gyroscope::new(device_from_address(addr)?)
    }

    /// Creates a new gyroscope reader from a SPI device.
    ///
    /// # Arguments
    /// * `dev`: The SPI device.
    pub fn new(mut dev: Spidev) -> Result<Self, crate::Error<io::Error>> {
        init(&mut dev, crate::LIS3MDL_WHO_AM_I, 0x3D)?;
        // Enable the gyroscope
        write_reg(&mut dev, crate::LSM6DSL_CTRL2_G, 0b10011100)?; // ODR 3.3 kHz, 2000 dps
        Ok(Self(dev))
    }

    /// Read the raw gyroscope values.
    pub fn read(&mut self) -> Result<(i32, i32, i32), crate::Error<io::Error>> {
        let x = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTX_L_G,
            crate::LSM6DSL_OUTX_H_G,
        )?;
        let y = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTY_L_G,
            crate::LSM6DSL_OUTY_H_G,
        )?;
        let z = read_axis(
            &mut self.0,
            crate::LSM6DSL_OUTZ_L_G,
            crate::LSM6DSL_OUTZ_H_G,
        )?;
        Ok((x, y, z))
    }
}
