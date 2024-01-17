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
mod interface;
mod traits;

use crate::display::interface::DisplayInterface;
use color::OctColor;
pub use display::InkyFrameDisplay;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};
pub use traits::IsBusy;

use self::command::Command;

/// Width of the display
pub const WIDTH: u32 = 600;
/// Height of the display
pub const HEIGHT: u32 = 448;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: OctColor = OctColor::White;

/// Epd5in65f driver
///
pub struct InkyFrame5_7<SPI, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, DC, RST>,
    /// Background Color
    color: OctColor,
}

impl<SPI, DC, RST> InkyFrame5_7<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    pub const WIDTH: u32 = WIDTH;
    pub const HEIGHT: u32 = HEIGHT;

    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        busy_signal: &mut impl IsBusy,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(dc, spi, rst);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut inky_frame = InkyFrame5_7 { interface, color };
        inky_frame.init(busy_signal)?;

        Ok(inky_frame)
    }

    fn init(&mut self, busy_signal: &mut impl IsBusy) -> Result<(), SPI::Error> {
        self.interface.reset(busy_signal);
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
        self.cmd_with_data(Command::VcomAndDataIntervalSetting, &[0x37])
    }

    pub fn power_off(&mut self) -> Result<(), SPI::Error> {
        // self.interface.wait_until_idle(100, delay);
        self.interface.cmd(Command::PowerOff)
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
        self.interface.data_x_times(bg, WIDTH / 2 * HEIGHT)?;
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

    fn command(&mut self, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(command)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(data)
    }

    fn cmd_with_data(&mut self, command: Command, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(command, data)
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

    fn update_vcom(&mut self) -> Result<(), SPI::Error> {
        let bg_color = (self.color.get_nibble() & 0b111) << 5;
        self.cmd_with_data(Command::VcomAndDataIntervalSetting, &[0x17 | bg_color])?;
        Ok(())
    }

    fn busy_wait(&mut self, busy_signal: &mut impl IsBusy) {
        self.interface.wait_until_idle(busy_signal)
    }
}
