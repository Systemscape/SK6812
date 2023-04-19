use embedded_hal::digital::OutputPin;
use embedded_hal::delay::DelayUs;

use crate::RGBW;


/// SK6812 struct holding a `Delay` and the pin to which the LEDs are connected
pub struct Sk6812<Delay: DelayUs, Pin: OutputPin> {
    delay: Delay,
    pin: Pin,
}

impl<Delay: DelayUs, Pin: OutputPin> Sk6812<Delay, Pin> {
    /// Construct an instance from a [`Delay`] and [`Pin`]
    pub fn new(delay: Delay, pin: Pin) -> Self {
        Self { delay, pin }
    }

    /// Write the RGBW colors stored in `iter` to the LEDs
    pub fn write(&mut self, iter: impl Iterator<Item = RGBW>) {
        for RGBW { r, g, b, a } in iter {
            self.write_byte(r);
            self.write_byte(g);
            self.write_byte(b);
            self.write_byte(a.0);
        }
        // Send reset code after writing all bytes
        self.delay.delay_us(90);
    }

    /// Write a single byte to the LED according to the specification
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            // If the MSB is 1 (i.e. masked byte non-zero), send the sequence for 1 and vice versa
            if (data & 0x80) != 0 {
                self.delay.delay_us(1);
                self.pin.set_high().ok();
                self.delay.delay_us(2); // Send long HIGH pulse
                self.pin.set_low().ok();
            } else {
                self.delay.delay_us(1);
                self.pin.set_high().ok();
                self.delay.delay_us(1); // Send short HIGH pulse
                self.pin.set_low().ok();
                self.delay.delay_us(1);
            }
            // Shift 1 bit left so the next bit is the new MSB
            data <<= 1;
        }
    }
}
