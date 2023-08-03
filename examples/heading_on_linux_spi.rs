use std::error::Error;
use std::f64;
use std::thread;
use std::time::{Duration, Instant};

use berryimu;

const G_GAIN: f64 = 0.070; // [deg/s/LSB] If you change the dps for gyro, you need to update this value accordingly
const AA: f64 = 0.40; // Complementary filter constant

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut accelerometer = berryimu::spi::Accelerometer::new_from_address("/dev/spidev0.0")?;
    let mut gyroscope = berryimu::spi::Gyroscope::new_from_address("/dev/spidev0.0")?;
    let mut last_instant = Instant::now();
    let mut gyro_x_angle = 0.0;
    let mut gyro_y_angle = 0.0;
    let mut gyro_z_angle = 0.0;
    let mut cf_angle_x = 0.0;
    let mut cf_angle_y = 0.0;

    loop {
        let (acc_x, acc_y, acc_z) = accelerometer.read()?;
        let (gyr_x, gyr_y, gyr_z) = gyroscope.read()?;

        let elapsed = last_instant.elapsed().as_secs_f64();
        last_instant = Instant::now();

        // Convert gyro raw to degrees per second
        let rate_gyr_x = gyr_x * G_GAIN;
        let rate_gyr_y = gyr_y * G_GAIN;
        let rate_gyr_z = gyr_z * G_GAIN;

        // Calculate the angles from the gyro.
        gyro_x_angle += rate_gyr_x * LP;
        gyro_y_angle += rate_gyr_y * LP;
        gyro_z_angle += rate_gyr_z * LP;

        // Convert Accelerometer values to degrees
        let acc_x_angle = 180.0 * acc_y.atan2(acc_z) / f64::consts::PI;
        let mut acc_y_angle = 180.0 * (acc_z.atan2(acc_x) + f64::consts::PI) / f64::consts::PI;

        // convert the values to -180 and +180
        if acc_y_angle > 90 {
            acc_y_angle -= 270.0;
        } else {
            acc_y_angle += 90.0;
        }

        // Complementary filter used to combine the accelerometer and gyro values.
        cf_angle_x = AA * (cf_angle_x + rate_gyr_x * LP) + (1 - AA) * acc_x_angle;
        cf_angle_y = AA * (cf_angle_y + rate_gyr_y * LP) + (1 - AA) * acc_y_angle;

        println!("{cf_angle_x:.2}");

        // Sleep for 25ms
        thread::sleep(Duration::from_millis(25));
    }
}
