# BerryIMU

[![crates.io](https://img.shields.io/crates/v/berryimu.svg)](https://crates.io/crates/berryimu)
[![Released API docs](https://docs.rs/berryimu/badge.svg)](https://docs.rs/berryimu)

A pure-rust library for interfacing with [BerryIMU v3](https://ozzmaker.com/product/berryimu-accelerometer-gyroscope-magnetometer-barometricaltitude-sensor/).

At the moment, this is just enough functionality to support [what's demoed in part 2 of the raspberry pi guide](https://ozzmaker.com/compass2/), as this is all I needed for building a robot. Specifically, this supports reading the accelerometer and magnetometer with fixed settings on linux. Pull requests to add more functionality are welcome.

## Example

This will print out the tilt-compensated heading on linux:

```rust
use std::error::Error;
use std::f64;
use std::thread;
use std::time::Duration;

use berryimu;

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut accelerometer = berryimu::Accelerometer::new_on_linux("/dev/i2c-1")?;
    let mut magnetometer = berryimu::Magnetometer::new_on_linux("/dev/i2c-1")?;

    loop {
        let (acc_x, acc_y, acc_z) = accelerometer.read()?;
        let (mag_x, mag_y, mag_z) = magnetometer.read()?;

        // Normalize accelerometer raw values.
        let acc_x_norm =
            (acc_x as f64) / ((acc_x * acc_x + acc_y * acc_y + acc_z * acc_z) as f64).sqrt();
        let acc_y_norm =
            (acc_y as f64) / ((acc_x * acc_x + acc_y * acc_y + acc_z * acc_z) as f64).sqrt();

        //Calculate pitch and roll
        let pitch = acc_x_norm.asin();
        let roll = -((acc_y_norm / pitch.cos()).asin());

        // Calculate the new tilt compensated values
        // The compass and accelerometer are oriented differently on the the BerryIMUv1, v2 and v3.
        // This needs to be taken into consideration when performing the calculations.
        // X compensation
        let mag_x_comp = (mag_x as f64) * pitch.cos() + (mag_z as f64) * pitch.sin();
        // Y compensation
        let mag_y_comp = (mag_x as f64) * roll.sin() * pitch.sin() + (mag_y as f64) * roll.cos()
            - (mag_z as f64) * roll.sin() * pitch.cos();

        // Calculate heading in degrees
        let mut heading = 180.0 * mag_y_comp.atan2(mag_x_comp) / f64::consts::PI;
        if heading < 0.0 {
            heading += 360.0;
        }

        println!("{heading:.2f}");

        // Sleep for 25ms
        thread::sleep(Duration::from_millis(25));
    }
}

```