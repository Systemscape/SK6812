use embedded_hal::digital::OutputPin;
use embedded_hal::delay::DelayUs;

use crate::RGBW;

/// SK6812 struct holding a `Delay` and the pin to which the LEDs are connected
pub struct Sk6812<Pin: OutputPin> {
    pin: Pin,
}

impl<Pin: OutputPin> Sk6812<Pin> {
    /// Construct an instance from a [`Delay`] and [`Pin`]
    pub fn new(pin: Pin) -> Self {
        Self { pin }
    }

    /// Write the RGBW colors stored in `iter` to the LEDs
    pub fn write(&mut self, delay: &mut impl DelayUs, iter: impl IntoIterator<Item = RGBW>) {
        for RGBW { r, g, b, a } in iter {
            self.write_byte(delay, r);
            self.write_byte(delay, g);
            self.write_byte(delay, b);
            self.write_byte(delay, a.0);
        }
        // Send reset code after writing all bytes
        let _ = delay.delay_us(90);
    }

    /// Write a single byte to the LED according to the specification
    fn write_byte(&mut self, delay: &mut impl DelayUs, mut data: u8) {
        for _ in 0..8 {
            // If the MSB is 1 (i.e. masked byte non-zero), send the sequence for 1 (long HIGH, short LOW) and vice versa
            if (data & 0x80) != 0 {
                let _ = delay.delay_us(1);
                self.pin.set_high().ok();
                let _ = delay.delay_us(1); // Send long HIGH pulse
                let _ = delay.delay_us(1);
                let _ = delay.delay_us(1);
                self.pin.set_low().ok();
            } else {
                let _ = delay.delay_us(1);
                self.pin.set_high().ok();
                let _ = delay.delay_us(1); // Send short HIGH pulse
                self.pin.set_low().ok();
                let _ = delay.delay_us(1);
                let _ = delay.delay_us(1);
            }
            // Shift 1 bit left so the next bit is the new MSB
            data <<= 1;
        }
    }
}
