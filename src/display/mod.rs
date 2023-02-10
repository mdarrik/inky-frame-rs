pub mod color;
/**
 * The display driver for the inky frame uc8159
 * A lot of code is modified from https://github.com/caemor/epd-waveshare
 * Specifically the epd5in65f which seems to be very similar in function
 * to the UC8159 driver of the inky frame.
 * ISC: https://github.com/caemor/epd-waveshare/blob/main/License.md
 *
 */
mod command;
mod display;
mod interface;
mod traits;

use crate::display::interface::DisplayInterface;
use color::OctColor;
pub use display::Display5in65f;
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use self::command::Command;

/// Width of the display
pub const WIDTH: u32 = 600;
/// Height of the display
pub const HEIGHT: u32 = 448;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: OctColor = OctColor::White;

/// Epd5in65f driver
///
pub struct Epd5in65f<SPI, CS, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, DC, RST, DELAY>,
    /// Background Color
    color: OctColor,
}

impl<SPI, CS, DC, RST, DELAY> Epd5in65f<SPI, CS, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    pub const WIDTH: u32 = WIDTH;
    pub const HEIGHT: u32 = HEIGHT;

    pub fn new(
        spi: &mut SPI,
        cs: CS,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, dc, rst);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd5in65f { interface, color };
        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.reset(delay);

        self.cmd_with_data(spi, Command::PanelSetting, &[0xEF, 0x08])?;
        self.cmd_with_data(spi, Command::PowerSetting, &[0x37, 0x00, 0x23, 0x23])?;
        self.cmd_with_data(spi, Command::PowerOffSequenceSetting, &[0x00])?;
        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0xC7, 0xC7, 0x1D])?;
        self.cmd_with_data(spi, Command::PllControl, &[0x3C])?;
        self.cmd_with_data(spi, Command::TemperatureSensor, &[0x00])?;
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x37])?;
        self.cmd_with_data(spi, Command::TconSetting, &[0x22])?;
        self.send_resolution(spi)?;
        self.cmd_with_data(spi, Command::FlashMode, &[0xAA])?;

        delay.delay_ms(100);

        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x37])
    }

    pub fn power_off(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.wait_until_idle(100, delay);
        self.interface.cmd(spi, Command::PowerOff)
    }

    pub fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    pub fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])
    }

    pub fn update_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
    ) -> Result<(), SPI::Error> {
        self.busy_wait(delay);
        self.update_vcom(spi)?;
        self.send_resolution(spi)?;
        self.cmd_with_data(spi, Command::DataStartTransmission1, buffer)
    }

    pub fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.busy_wait(delay);
        self.command(spi, Command::PowerOn)?;
        self.busy_wait(delay);
        self.command(spi, Command::DisplayRefresh)?;
        self.busy_wait(delay);
        self.command(spi, Command::PowerOff)?;
        self.busy_wait(delay);
        Ok(())
    }

    pub fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, delay, buffer)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    pub fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        let bg = OctColor::colors_byte(self.color, self.color);
        self.busy_wait(delay);
        self.update_vcom(spi)?;
        self.send_resolution(spi)?;
        self.command(spi, Command::DataStartTransmission1)?;
        self.display_frame(spi, delay)?;
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

    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn send_data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(spi, data)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self::WIDTH;
        let h = self::HEIGHT;

        self.command(spi, Command::TconResolution)?;
        self.send_data(spi, &[(w >> 8) as u8])?;
        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }

    fn update_vcom(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let bg_color = (self.color.get_nibble() & 0b111) << 5;
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x17 | bg_color])?;
        Ok(())
    }

    fn busy_wait(&mut self, delay: &mut DELAY) {
        self.interface.wait_until_idle(100, delay)
    }
}
