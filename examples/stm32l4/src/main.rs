#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;

use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::rcc::ClockSrc;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;

use embedded_hal::blocking::delay::DelayUs;

use {defmt_rtt as _, panic_probe as _};

use sk6812::new_rgbw;
use sk6812::sk6812_async::{Sk6812Spi};
use sk6812::sk6812_blocking::Sk6812;

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

    let spi = Spi::new_txonly_nosck(
        p.SPI1,
        p.PA7,
        p.DMA1_CH3,
        NoDma,
        Hertz(3_000_000),
        Config::default(),
    );

    //let mut pin = Output::new(p.PA7, Level::High, Speed::Low);

    let mut led: Sk6812Spi<_, { 16 * 11 }> = Sk6812Spi::new(spi);
    
    //let mut led = Sk6812::new(pin);

    let mut delay = embassy_time::Delay;
    let _ = delay.delay_us(9000_u32);

    let mut val = 0_u8;
    let mut val2 = 128_u8;
    loop {
        let iter = [
            new_rgbw(val, val, val, val),
            new_rgbw(val2, val2, val2, val2),
            new_rgbw(val, val, val, val),
            new_rgbw(val2, val2, val2, val2),
            new_rgbw(val, val, val, val),
            new_rgbw(val2, val2, val2, val2),
            new_rgbw(val, val, val, val),
            new_rgbw(val2, val2, val2, val2),
            new_rgbw(val, val, val, val),
            new_rgbw(val2, val2, val2, val2),
        ];
        //led.write(&mut embassy_time::Delay, iter);
        led.write(iter).await.unwrap();

        val = val.overflowing_add(1).0;
        val2 = val2.overflowing_add(1).0;
        
        let _ = delay.delay_us(9000_u32);

        //info!("Value: {}", val);
    }

    //let mut spi = BlockingAsync::new(spi);
    /*
        let write: [u8; 10] = [0x0A; 10];
        loop {
            led.write(&write).await.unwrap();

            info!("written");
        }
    */
}
