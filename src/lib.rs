#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![feature(type_alias_impl_trait)]

/// Async implementation for use with embedded-hal-async
#[cfg(feature = "async")]
pub mod sk6812_async;

/// Blocking implementation for use with embedded-hal
#[cfg(feature = "blocking")]
#[deprecated(
    note = "Seems to be not working with SK6812 because a us delay is too long. Use the Sk6812Spi instead."
)]
pub mod sk6812_blocking;

/// RGBW type
pub type RGBW = smart_leds::RGBW<u8>;

/// Convenience function to create an [`RGBW`] from 4 [`u8`]s
pub fn new_rgbw(r: u8, g: u8, b: u8, w: u8) -> RGBW {
    RGBW {
        r,
        g,
        b,
        a: smart_leds::White(w),
    }
}
