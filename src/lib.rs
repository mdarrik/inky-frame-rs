#![no_std]

#[cfg(feature = "display")]
pub mod display;

#[cfg(feature = "shift_register")]
pub mod shift_register;

pub use display::InkyFrameDisplay;
