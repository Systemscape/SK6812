//! embassy wait
//!
//! This is an example of asynchronously `Wait`ing for a pin state to change.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use esp32c3_hal::{
    clock::ClockControl,
    embassy,
    gpio::{Gpio1, Output, PushPull},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc, IO,
};
use esp_backtrace as _;
use static_cell::StaticCell;

use sk6812::{new_rgbw, sk6812_async, sk6812_blocking};

#[embassy_executor::task]
async fn flash_leds(pin: Gpio1<Output<PushPull>>) {
    let mut led = sk6812_async::Sk6812::new(pin).await;

    esp_println::println!("Waiting...");
    let mut val = 0_u8;
    loop {
        let iter = [new_rgbw(val, val, val, 0)];
        led.write(&mut embassy_time::Delay, iter).await;

        //val = val.overflowing_add(1).0;

        embassy_time::Delay.delay_ms(100_u8);

        esp_println::println!("Value: {val}");
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
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

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // GPIO 1 as output
    let output_pin = io.pins.gpio1.into_push_pull_output();

    // Async requires the GPIO interrupt to wake futures
    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::GPIO,
        esp32c3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let mut led = sk6812_blocking::Sk6812::new(output_pin);

    esp_println::println!("Waiting...");
    let mut val = 0_u8;
    loop {
        let iter = [
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            new_rgbw(0,0,0,0),
            ];
        led.write(&mut embassy_time::Delay, iter);

        //val = val.overflowing_add(1).0;

        embassy_time::Delay.delay_ms(10_u8);

        esp_println::println!("Value: {val}");
    }

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(flash_leds(output_pin)).ok();
    });
}
