use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;

use crate::RGBW;

/// SK6812 struct holding a `Delay` and the pin to which the LEDs are connected
pub struct Sk6812<Pin: OutputPin> {
    pin: Pin,
}

impl<Pin: OutputPin> Sk6812<Pin> {
    /// Construct an instance from a [`Delay`] and [`Pin`]
    pub async fn new(pin: Pin) -> Self {
        Self { pin }
    }

    /// Write the RGBW colors stored in `iter` to the LEDs
    pub async fn write(&mut self, delay: &mut impl DelayUs, iter: impl IntoIterator<Item = RGBW>) {
        for RGBW { r, g, b, a } in iter {
            self.write_byte(delay, r).await;
            self.write_byte(delay, g).await;
            self.write_byte(delay, b).await;
            self.write_byte(delay, a.0).await;
        }
        // Send reset code after writing all bytes
        let _ = delay.delay_us(90).await;
    }

    /// Write a single byte to the LED according to the specification
    async fn write_byte(&mut self, delay: &mut impl DelayUs, mut data: u8) {
        for _ in 0..8 {
            // If the MSB is 1 (i.e. masked byte non-zero), send the sequence for 1 and vice versa
            if (data & 0x80) != 0 {
                let _ = delay.delay_us(1).await;
                self.pin.set_high().ok();
                let _ = delay.delay_us(2).await; // Send long HIGH pulse
                self.pin.set_low().ok();
            } else {
                let _ = delay.delay_us(1).await;
                self.pin.set_high().ok();
                let _ = delay.delay_us(1).await; // Send short HIGH pulse
                self.pin.set_low().ok();
                let _ = delay.delay_us(1).await;
            }
            // Shift 1 bit left so the next bit is the new MSB
            data <<= 1;
        }
    }
}
