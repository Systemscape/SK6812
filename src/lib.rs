#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![feature(type_alias_impl_trait)]

use smart_leds::RGB;

/// Async implementation for use with embedded-hal-async
#[cfg(feature = "async")]
pub mod sk6812_async;

/// Blocking implementation
#[cfg(feature = "blocking")]
pub mod sk6812_blocking;

/// RGBW type
pub type RGBW = smart_leds::RGBW<u8>;

/// Convenience function
pub fn new_rgbw(r: u8, g: u8, b: u8, w: u8) -> RGBW {
    RGBW {
        r,
        g,
        b,
        a: smart_leds::White(w),
    }
}
