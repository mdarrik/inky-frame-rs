#![no_std]

#[cfg(feature = "display")]
pub mod display;

#[cfg(feature = "shift_register")]
pub mod shift_register;

pub use display::InkyFrameDisplay;

/// Trait for determining if the e-ink display is busy
/// This could be the busy_pin for the inky impression or the shift_register for inky frame
pub trait IsBusy {
    fn is_busy(&mut self) -> bool;
}
