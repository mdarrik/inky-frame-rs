#![no_std]

#[cfg(feature = "display")]
pub mod display;

pub mod shift_register;

pub use display::InkyFrameDisplay;
