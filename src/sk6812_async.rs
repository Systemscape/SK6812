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

use embedded_hal_async::spi::{ErrorType, SpiBusWrite};

const PATTERNS: [u8; 4] = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];

/// N = 12 * NUM_LEDS
pub struct Sk6812Spi<SPI: SpiBusWrite<u8>, const N: usize> {
    spi: SPI,
    data: [u8; N],
}

impl<SPI: SpiBusWrite<u8>, const N: usize> Sk6812Spi<SPI, N> {
    /// Create new
    pub fn new(spi: SPI) -> Self {
        Self { spi, data: [0; N] }
    }

    /// Write RGBW values
    pub async fn write(
        &mut self,
        iter: impl IntoIterator<Item = RGBW>,
    ) -> Result<(), <SPI as ErrorType>::Error> {
        for (led_bytes, RGBW { r, g, b, a }) in self.data.chunks_mut(18).zip(iter) {
            for (i, mut color) in [r, g, b, a.0].into_iter().enumerate() {
                for ii in 0..4 {
                    led_bytes[i * 4 + ii] = PATTERNS[((color & 0b1100_0000) >> 6) as usize];
                    color <<= 2;
                }
            }
        }
        self.spi.write(&self.data).await?;
        let blank = [0_u8; 140];
        self.spi.write(&blank).await
    }
}
