pub mod color;
/**
 * The display driver for the inky frame uc8159
 * A lot of code is modified from https://github.com/caemor/epd-waveshare
 * Specifically the epd5in65f which seems to be very similar in function
 * to the UC8159 driver of the inky frame.
 * ISC: https://github.com/caemor/epd-waveshare/blob/main/License.md
 *
 * Also includes code adapted from pimoroni's uc8159 driver https://github.com/pimoroni/pimoroni-pico/blob/main/drivers/uc8159/uc8159.cpp
 * MIT License: https://github.com/pimoroni/pimoroni-pico/blob/main/LICENSE
 *
 * Some code also adapted from an existing uc8159 Rust driver
 * https://github.com/dflemstr/uc8159
 *
 */
mod command;
mod display;

// use crate::display::interface::DisplayInterface;
use self::command::Command;
use color::OctColor;
pub use display::InkyFrameDisplay;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};

/// Width of the display
pub const WIDTH: u32 = 600;
/// Height of the display
pub const HEIGHT: u32 = 448;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: OctColor = OctColor::White;

/// Driver for the Inky 5.7" 7 color e-ink display.
/// This should cover both the inky frame and inky impression drivers
/// Both are based off of the
pub struct InkyFrame5_7<SPI, DC, RST, DELAY> {
    /// SPI Device - used for writing data to the display
    spi: SPI,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Resetting
    rst: RST,

    /// Connection Interface
    /// Background Color
    color: OctColor,
    delay: DELAY,
}

impl<SPI, DC, RST, DELAY> InkyFrame5_7<SPI, DC, RST, DELAY>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: embedded_hal::delay::DelayNs,
{
    pub const WIDTH: u32 = WIDTH;
    pub const HEIGHT: u32 = HEIGHT;

    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        delay: DELAY,
        busy_signal: &mut impl IsBusy,
    ) -> Result<Self, SPI::Error> {
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut inky_frame = InkyFrame5_7 {
            spi,
            dc,
            rst,
            color,
            delay,
        };
        inky_frame.init(busy_signal)?;

        Ok(inky_frame)
    }

    fn init(&mut self, busy_signal: &mut impl IsBusy) -> Result<(), SPI::Error> {
        self.reset(busy_signal);
        self.busy_wait(busy_signal);
        self.cmd_with_data(Command::PanelSetting, &[0xEF, 0x08])?;
        self.cmd_with_data(Command::PowerSetting, &[0x37, 0x00, 0x23, 0x23])?;
        self.cmd_with_data(Command::PowerOffSequenceSetting, &[0x00])?;
        self.cmd_with_data(Command::BoosterSoftStart, &[0xC7, 0xC7, 0x1D])?;
        self.cmd_with_data(Command::PllControl, &[0x3C])?;
        self.cmd_with_data(Command::TemperatureSensor, &[0x00])?;
        self.cmd_with_data(Command::VcomAndDataIntervalSetting, &[0x37])?;
        self.cmd_with_data(Command::TconSetting, &[0x22])?;
        self.send_resolution()?;
        self.cmd_with_data(Command::FlashMode, &[0xAA])?;
        self.delay.delay_ms(10);
        self.cmd_with_data(Command::VcomAndDataIntervalSetting, &[0x37])
    }

    pub fn power_off(&mut self) -> Result<(), SPI::Error> {
        self.command(Command::PowerOff)
    }

    pub fn wake_up(&mut self, busy_signal: &mut impl IsBusy) -> Result<(), SPI::Error> {
        self.init(busy_signal)
    }

    pub fn sleep(&mut self) -> Result<(), SPI::Error> {
        self.cmd_with_data(Command::DeepSleep, &[0xA5])
    }

    pub fn update_frame(
        &mut self,
        busy_signal: &mut impl IsBusy,
        buffer: &[u8],
    ) -> Result<(), SPI::Error> {
        self.busy_wait(busy_signal);
        self.update_vcom()?;
        self.send_resolution()?;
        self.cmd_with_data(Command::DataStartTransmission1, buffer)?;
        self.command(Command::DataStop)
    }

    pub fn display_frame(&mut self, busy_signal: &mut impl IsBusy) -> Result<(), SPI::Error> {
        self.busy_wait(busy_signal);
        self.command(Command::PowerOn)?;
        self.busy_wait(busy_signal);
        self.command(Command::DisplayRefresh)?;
        self.busy_wait(busy_signal);
        self.command(Command::PowerOff)?;
        self.busy_wait(busy_signal);
        Ok(())
    }

    pub fn update_and_display_frame(
        &mut self,
        busy_signal: &mut impl IsBusy,
        buffer: &[u8],
    ) -> Result<(), SPI::Error> {
        self.update_frame(busy_signal, buffer)?;
        self.display_frame(busy_signal)?;
        Ok(())
    }

    pub fn clear_frame(&mut self, busy_signal: &mut impl IsBusy) -> Result<(), SPI::Error> {
        let bg = OctColor::colors_byte(self.color, self.color);
        self.busy_wait(busy_signal);
        self.update_vcom()?;
        self.send_resolution()?;
        self.command(Command::DataStartTransmission1)?;
        self.data_x_times(bg, WIDTH / 2 * HEIGHT)?;
        self.display_frame(busy_signal)?;
        Ok(())
    }

    pub fn set_background_color(&mut self, color: OctColor) {
        self.color = color;
    }

    pub fn width(&self) -> u32 {
        WIDTH
    }

    pub fn height(&self) -> u32 {
        HEIGHT
    }

    /// update the vcom setting to related to the default background color
    fn update_vcom(&mut self) -> Result<(), SPI::Error> {
        let bg_color = (self.color.get_nibble() & 0b111) << 5;
        self.cmd_with_data(Command::VcomAndDataIntervalSetting, &[0x17 | bg_color])?;
        Ok(())
    }

    /// reset the display using the reset pin
    pub fn reset(&mut self, busy_signal: &mut impl IsBusy) {
        let _ = self.rst.set_low();
        self.delay.delay_ms(10);
        let _ = self.rst.set_high();
        self.delay.delay_ms(10);
        self.busy_wait(busy_signal);
    }

    // helpers for sending data

    /// Write's a command to the e-ink display.
    /// Pairs with send_data to interact with the device.
    fn command(&mut self, command: Command) -> Result<(), SPI::Error> {
        // low for commands
        let _ = self.dc.set_low();

        // Transfer the command over spi
        self.write(&[command.address()])
    }

    /// Basic function for sending an array of u8-values of data over spi
    /// Enables direct interaction with the device with the help of [command()](InkyFrame5_7::command())
    fn send_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();

        for val in data.iter().copied() {
            // Transfer data one u8 at a time over spi
            self.write(&[val])?;
        }

        Ok(())
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](InkyFrame5_7::command())
    pub(crate) fn data_x_times(&mut self, val: u8, repetitions: u32) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(&[val])?;
        }
        Ok(())
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    fn cmd_with_data(&mut self, command: Command, data: &[u8]) -> Result<(), SPI::Error> {
        self.command(command)?;
        self.send_data(data)
    }

    fn send_resolution(&mut self) -> Result<(), SPI::Error> {
        let w = self::WIDTH;
        let h = self::HEIGHT;

        self.command(Command::TconResolution)?;
        self.send_data(&[(w >> 8) as u8])?;
        self.send_data(&[w as u8])?;
        self.send_data(&[(h >> 8) as u8])?;
        self.send_data(&[h as u8])
    }

    fn busy_wait(&mut self, busy_signal: &mut impl IsBusy) {
        while busy_signal.is_busy() {}
    }

    /// spi write helper/abstraction function
    fn write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        // transfer spi data
        self.spi.write(data)?;
        Ok(())
    }
}

/// Trait for determining if the e-ink display is busy
/// This could be the busy_pin for the inky impression or the shift_register for inky frame
pub trait IsBusy {
    fn is_busy(&mut self) -> bool;
}
