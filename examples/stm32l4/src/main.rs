#![no_std]
#![no_main]
#![deny(warnings)]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    dma::NoDma,
    rcc::ClockSrc,
    spi::{Config, Spi},
    time::Hertz,
};

use embassy_time::Delay;
use embedded_hal_async::delay::DelayUs;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

use sk6812::new_rgbw;
use sk6812::sk6812_async::Sk6812Spi;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

pub type SpiType<'d> = embassy_stm32::spi::Spi<
    'd,
    embassy_stm32::peripherals::SPI1,
    embassy_stm32::peripherals::DMA1_CH3,
    embassy_stm32::dma::NoDma,
>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    info!("Tick Hz is {}", embassy_time::TICK_HZ);

    // Create tx only spi instance on pin A8 (PA_7) of the nucleo board
    let spi = singleton!(Spi::new_txonly_nosck(
        p.SPI1,
        p.PA7,
        p.DMA1_CH3,
        NoDma,
        Hertz(3_000_000),
        Config::default(),
    ));

    spawner.spawn(led_task(spi)).ok();
    spawner.spawn(print_task()).ok();
}

#[embassy_executor::task]
async fn led_task(spi: &'static mut SpiType<'static>) {
    // Create an instance of the led for 9 LEDs on the strip
    let mut led: Sk6812Spi<_, { 9 * 16 }> = Sk6812Spi::new(spi);

    // Counter to light up the LEDs one after the other
    let mut counter = 0;

    info!("Entering led_task loop...");
    loop {
        // Array of colors, each represents a single LED
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

        // First iteration no LED, second iteration first LED, third iteration second LED etc.
        let colors = all_colors.iter().enumerate().map(|(i, color)| {
            if i < counter {
                *color
            } else {
                new_rgbw(0, 0, 0, 0)
            }
        });

        // Output the color iterator to the led strip
        led.write(colors).await.expect("Write to led");

        counter += 1;
        if counter > 9 {
            counter = 0;
        }
        DelayUs::delay_ms(&mut Delay {}, 1000u32).await;
    }
}

#[embassy_executor::task]
async fn print_task() {
    // Counter to light up the LEDs one after the other
    let mut counter = 0;

    info!("Entering print_task loop...");
    loop {
        info!("print_task look counter: {}", counter);
        counter += 1;

        DelayUs::delay_ms(&mut Delay {}, 200u32).await;
    }
}
