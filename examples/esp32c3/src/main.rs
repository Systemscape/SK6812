//! embassy wait
//!
//! This is an example of asynchronously `Wait`ing for a pin state to change.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::{Executor, Spawner};
use esp32c3_hal::{
    clock::ClockControl,
    dma::{DmaPriority, ChannelTx, ChannelRx},
    embassy,
    gdma::{Gdma, Channel0TxImpl, Channel0RxImpl, SuitablePeripheral0},
    gpio::{Gpio1, Output, PushPull},
    peripherals::{Peripherals, self},
    prelude::*,
    timer::TimerGroup,
    Rtc, Spi, IO, spi::{dma::SpiDma, FullDuplexMode},
};
use esp_backtrace as _;
use static_cell::StaticCell;

use sk6812::{new_rgbw, sk6812_async, sk6812_async::Sk6812Spi, sk6812_blocking};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

pub type SpiType<'d> = SpiDma<
    'd,
    esp32c3_hal::peripherals::SPI2,
    ChannelTx<'d, Channel0TxImpl, esp32c3_hal::gdma::Channel0>,
    ChannelRx<'d, Channel0RxImpl, esp32c3_hal::gdma::Channel0>,
    SuitablePeripheral0,
    FullDuplexMode,
>;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    embassy::init(
        &clocks,
        esp32c3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    //let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // GPIO 1 as output
    //let output_pin = io.pins.gpio1.into_push_pull_output();

    // Async requires the GPIO interrupt to wake futures
    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::GPIO,
        esp32c3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);

    let dma_channel = dma.channel0;
    let descriptors = singleton!([0u32; 8 * 3]);
    let rx_descriptors = singleton!([0u32; 8 * 3]);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut spi = Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        io.pins.gpio2,
        io.pins.gpio1,
        3800u32.kHz(),
        esp32c3_hal::spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .with_dma(dma_channel.configure(
        false,
        descriptors,
        rx_descriptors,
        DmaPriority::Priority0,
    ));

    //let mut led = sk6812_blocking::Sk6812::new(output_pin);
    let mut led: Sk6812Spi<_, { 16 * 10 }> = Sk6812Spi::new(spi);

    esp_println::println!("Waiting...");
    let mut val = 0_u8;
    loop {
        let iter = [
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
            new_rgbw(0, 0, 0, 0),
        ];
        //led.write(&mut embassy_time::Delay, iter);
        led.write(iter).await;

        //val = val.overflowing_add(1).0;

        embassy_time::Delay.delay_ms(10_u8);

        esp_println::println!("Value: {val}");
    }

}
