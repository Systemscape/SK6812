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
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;

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

    info!("Tick Hz is {}", embassy_time::TICK_HZ);

    //let mut pin = Output::new(p.PA2, Level::High, Speed::Low);

    let mut led: Sk6812Spi<_, { 16 * 10 }> = Sk6812Spi::new(spi);
    
    //let mut led = Sk6812::new(pin);

    let mut rng = embassy_stm32::rng::Rng::new(p.RNG);

    let mut buf = [0u8; 36];

    let mut delay = embassy_time::Delay;

    let mut val = 0_u8;
    let mut val2 = 128_u8;
    loop {
        unwrap!(rng.async_fill_bytes(&mut buf).await);
        let iter = [
            new_rgbw(buf[0], buf[1], buf[2],    0),//buf[3]),
            new_rgbw(buf[4], buf[5], buf[6],    0),//buf[7]),
            new_rgbw(buf[8], buf[9], buf[10],   0),//buf[11]),
            new_rgbw(buf[12], buf[13], buf[14], 0),//buf[15]),
            new_rgbw(buf[16], buf[17], buf[18], 0),//buf[19]),
            new_rgbw(buf[20], buf[21], buf[22], 0),//buf[23]),
            new_rgbw(buf[24], buf[25], buf[26], 0),//buf[27]),
            new_rgbw(buf[28], buf[29], buf[20], 0),//buf[31]),
            new_rgbw(buf[32], buf[33], buf[34], 0),//buf[35]),
        ];
        //led.write(&mut embassy_time::Delay, iter);
        //led.write(&mut delay, iter);
        led.write(iter).await.unwrap();

        val = val.overflowing_add(1).0;
        val2 = val2.overflowing_add(1).0;
        
        if val > 10 {
            val = 0;
        }
        if val2 > 10 {
            val2 = 0;
        }
        delay.delay_ms(500_u32);
        //pin.toggle();

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
