use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;

use crate::RGBW;

/// SK6812 struct holding a `Delay` and the pin to which the LEDs are connected
pub struct Sk6812<Pin: OutputPin> {
    pin: Pin,
}

#[deprecated(
    note = "Seems to be not working with SK6812 because a us delay is too long. Use the Sk6812Spi instead."
)]
impl<Pin: OutputPin> Sk6812<Pin> {
    /// Construct an instance from an[`OutputPin`]
    pub fn new(pin: Pin) -> Self {
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
        delay.delay_us(90).await;
    }

    /// Write a single byte to the LED according to the specification
    async fn write_byte(&mut self, delay: &mut impl DelayUs, mut data: u8) {
        for _ in 0..8 {
            // If the MSB is 1 (i.e. masked byte non-zero), send the sequence for 1 and vice versa
            if (data & 0x80) != 0 {
                delay.delay_us(1).await;
                self.pin.set_high().ok();
                delay.delay_us(2).await; // Send long HIGH pulse
                self.pin.set_low().ok();
            } else {
                delay.delay_us(1).await;
                self.pin.set_high().ok();
                delay.delay_us(1).await; // Send short HIGH pulse
                self.pin.set_low().ok();
                delay.delay_us(1).await;
            }
            // Shift 1 bit left so the next bit is the new MSB
            data <<= 1;
        }
    }
}

use embedded_hal_async::spi::{ErrorType, SpiBusWrite};

// Bit patterns sent over SPI. 4 bit on SPI are used to represent one color bit
// So each bye in `patterns` encodes two bits of `data`
const PATTERNS: [u8; 4] = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];

/// Async SPI-based driver for SK6812 RGBW LEDs
/// `NUM_LEDS_X_16` is the number of RGBW LEDs on the strip times 16.
/// # Example:
/// ```ignore
/// use sk6812::sk6812_async::Sk6812Spi;
/// use embassy_stm32::spi::{Config, Spi};
/// // Initialize the board and obtain a Peripherals instance
/// let p: = embassy_stm32::init(Default::default());
/// // Create tx only spi instance on pin A8 (PA_7) of the nucleo board
/// let spi = Spi::new_txonly_nosck(
///     p.SPI1,
///     p.PA7,
///     p.DMA1_CH3,
///     NoDma,
///     Hertz(3_000_000),
///     Config::default(),
/// );
/// // Create an instance of the led for 9 LEDs on the strip
/// let mut led: Sk6812Spi<_, {9*16}> = Sk6812Spi::new(spi);
/// ```
pub struct Sk6812Spi<SPI: SpiBusWrite<u8>, const NUM_LEDS_X_16: usize> {
    spi: SPI,
    spi_buffer: [u8; NUM_LEDS_X_16],
}

impl<SPI: SpiBusWrite<u8>, const NUM_LEDS_X_16: usize> Sk6812Spi<SPI, NUM_LEDS_X_16> {
    /// Create instance using the given [`SpiBusWrite`]
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            spi_buffer: [0_u8; NUM_LEDS_X_16],
        }
    }

    /// Write the RGBW colors stored in `iter` to the LEDs
    pub async fn write(
        &mut self,
        iter: impl IntoIterator<Item = RGBW>,
    ) -> Result<(), <SPI as ErrorType>::Error> {
        // Iterate over all colors and get chunks of 4 byte * 4 byte
        // since each of the 4 colors (RGBW) has 1 byte
        // And on the SPI we represent 2 bits as 1 byte (so each color byte gets 4 spi bytes)
        for (spi_bytes, RGBW { r, g, b, a }) in self.spi_buffer.chunks_mut(4 * 4).zip(iter) {
            // Iterate over each color while keeping track of the index (1 byte color => 4 bytes on spi)
            for (idx_color, mut color) in [r, g, b, a.0].into_iter().enumerate() {
                // Iterate over 4 bit pairs (i.e. one byte of data)
                for idx_two_bits in 0..4 {
                    // Mask the highest two bit and shift them to the two lowest bits
                    let two_bits = (color & 0b1100_0000) >> 6;

                    // Send the pattern corresponding to the two bits
                    spi_bytes[idx_color * 4 + idx_two_bits] = PATTERNS[two_bits as usize];
                    color <<= 2;
                }
            }
        }
        self.spi.write(&self.spi_buffer).await?;
        self.flush().await.unwrap();

        Ok(())
    }

    /// Send zeros as reset code
    #[inline]
    async fn flush(&mut self) -> Result<(), <SPI as ErrorType>::Error> {
        // Should be > 300μs, so for an SPI Freq. of 3.8MHz, we have to send at least 1140 low bits or 140 low bytes
        let zero = [0_u8; 140];
        self.spi.write(&zero).await?;
        self.spi.flush();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    use crate::sk6812_async::Sk6812Spi;

    #[tokio::test]
    async fn led_all_off() {
        let expectations = [
            SpiTransaction::write_vec(vec![0b1000_1000; 16]),
            SpiTransaction::write_vec(vec![0_u8; 140]),
        ];
        let mut spi = SpiMock::new(&expectations);

        let mut led: Sk6812Spi<_, 16> = Sk6812Spi::new(&mut spi);

        let colors = [crate::new_rgbw(0, 0, 0, 0)];

        // Output the color iterator to the led strip
        led.write(colors).await.expect("Write to led");

        spi.done();
    }
}
