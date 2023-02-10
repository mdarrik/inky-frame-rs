#![no_std]

#[cfg(feature = "display")]
pub mod display;

pub use display::Display5in65f;
