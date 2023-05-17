#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayUs;
use esp32c3_hal::{
    clock::ClockControl, dma::DmaPriority, embassy, gdma::Gdma, peripherals::Peripherals,
    prelude::*, timer::TimerGroup, Rtc, Spi, IO,
};
use esp_backtrace as _;
use static_cell::StaticCell;

use sk6812::{new_rgbw, sk6812_async::Sk6812Spi};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

// Admittedly, this is a bit awkward. But there doesn't seem to be a better solution right now.
pub type SpiType<'d> = esp32c3_hal::spi::dma::SpiDma<
    'd,
    esp32c3_hal::soc::peripherals::SPI2,
    esp32c3_hal::dma::ChannelTx<
        'd,
        esp32c3_hal::dma::gdma::Channel0TxImpl,
        esp32c3_hal::dma::gdma::Channel0,
    >,
    esp32c3_hal::dma::ChannelRx<
        'd,
        esp32c3_hal::dma::gdma::Channel0RxImpl,
        esp32c3_hal::dma::gdma::Channel0,
    >,
    esp32c3_hal::gdma::SuitablePeripheral0,
    esp32c3_hal::spi::FullDuplexMode,
>;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    esp_println::println!("Init peripherals etc.");

    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
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

    // Async requires the GPIO interrupt to wake futures
    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::DMA_CH0,
        esp32c3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mosi = io.pins.gpio1;

    let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);
    let dma_channel = dma.channel0;

    let descriptors = singleton!([0u32; 8 * 3]);
    let rx_descriptors = singleton!([0u32; 8 * 3]);

    esp_println::println!("Init SPI");

    // Create a 300kHz SPI on the configured mosi pin
    let spi = singleton!(Spi::new_mosi_only(
        peripherals.SPI2,
        mosi,
        3000u32.kHz(),
        esp32c3_hal::spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .with_dma(dma_channel.configure(
        false,
        descriptors,
        rx_descriptors,
        DmaPriority::Priority0,
    )));

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(led_task(spi)).ok();
        spawner.spawn(print_task()).ok();
    });
}

#[embassy_executor::task]
async fn led_task(spi: &'static mut SpiType<'static>) {
    // Create an instance of the led for 9 LEDs on the strip
    let mut led: Sk6812Spi<_, { 9 * 16 }> = Sk6812Spi::new(spi);

    // Counter to light up the LEDs one after the other
    let mut counter = 0;

    esp_println::println!("Entering led_task loop...");
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

    esp_println::println!("Entering print_task loop...");
    loop {
        esp_println::println!("print_task look counter: {counter}");
        counter += 1;

        DelayUs::delay_ms(&mut Delay {}, 200u32).await;
    }
}
