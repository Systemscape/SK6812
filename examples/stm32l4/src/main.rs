#![no_std]
#![no_main]
#![deny(warnings)]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;

use embassy_stm32::rcc::ClockSrc;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use {defmt_rtt as _, panic_probe as _};

use sk6812::new_rgbw;
use sk6812::sk6812_async::Sk6812Spi;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // 72Mhz clock (16 / 1 * 18 / 4)
    // PLLSrc / SrcDiv * PLLMul / ClkDiv
    config.rcc.mux = ClockSrc::PLL(
        embassy_stm32::rcc::PLLSource::HSI16,
        embassy_stm32::rcc::PLLClkDiv::Div4,
        embassy_stm32::rcc::PLLSrcDiv::Div1,
        embassy_stm32::rcc::PLLMul::Mul18,
        Some(embassy_stm32::rcc::PLLClkDiv::Div6), // 48Mhz (16 / 1 * 18 / 6)
    );
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    info!("Tick Hz is {}", embassy_time::TICK_HZ);

    let spi = Spi::new_txonly_nosck(
        p.SPI1,
        p.PA7,
        p.DMA1_CH3,
        NoDma,
        Hertz(3_000_000),
        Config::default(),
    );
    let mut led: Sk6812Spi<_, { 16 * 10 }> = Sk6812Spi::new(spi);
    let mut delay = embassy_time::Delay;

    let mut counter = 0;

    loop {
        let all_colors = [
            new_rgbw(10, 0, 0, 0),
            new_rgbw(0, 10, 0, 0),
            new_rgbw(0, 0, 10, 0),
            new_rgbw(0, 0, 0, 10),
            new_rgbw(10, 0, 0, 0),
            new_rgbw(0, 10, 0, 0),
            new_rgbw(0, 0, 10, 0),
            new_rgbw(0, 0, 0, 10),
            new_rgbw(10, 10, 10, 10),
        ];

        let colors = all_colors.iter().enumerate().map(|(i, color)| 
        {
            if i < counter {
                *color
            } else {
                new_rgbw(0, 0, 0, 0)
            }
        });

        led.write(colors).await.unwrap();

        counter += 1;
        if counter > 9 {
            counter = 0;
        }

        delay.delay_ms(500_u32);
    }

}
