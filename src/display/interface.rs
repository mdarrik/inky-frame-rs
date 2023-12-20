use crate::display::traits::Command;
use core::marker::PhantomData;

use embedded_hal::{
    blocking::spi::Write,
    digital::v2::OutputPin,
};

use super::IsBusy;
/// Interface for the display
pub(crate) struct DisplayInterface<SPI, CS, DC, RST> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// Chip Select for SPI
    cs: CS,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Resetting
    rst: RST,
}

impl<SPI, CS, DC, RST> DisplayInterface<SPI, CS, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(cs: CS, dc: DC, rst: RST) -> Self {
        DisplayInterface {
            _spi: PhantomData::default(),
            cs,
            dc,
            rst,
        }
    }

    /// Basic function for sending [Commands](Command).
    ///
    /// Enables direct interaction with the device with the help of [data()](DisplayInterface::data())
    pub(crate) fn cmd<T: Command>(&mut self, spi: &mut SPI, command: T) -> Result<(), SPI::Error> {
        // low for commands
        let _ = self.dc.set_low();

        // Transfer the command over spi
        self.write(spi, &[command.address()])
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](Epd4in2::command())
    pub(crate) fn data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();

        for val in data.iter().copied() {
            // Transfer data one u8 at a time over spi
            self.write(spi, &[val])?;
        }

        Ok(())
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_data<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.cmd(spi, command)?;
        self.data(spi, data)
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    pub(crate) fn data_x_times(
        &mut self,
        spi: &mut SPI,
        val: u8,
        repetitions: u32,
    ) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(spi, &[val])?;
        }
        Ok(())
    }

    /// spi write helper/abstraction function
    fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // activate spi with cs low
        let _ = self.cs.set_low();

        // transfer spi data
        spi.write(data)?;

        // deactivate spi with cs high
        let _ = self.cs.set_high();

        Ok(())
    }

    /// waits until the device is not busy
    pub(crate) fn wait_until_idle(&mut self, busy_signal: &mut impl IsBusy) {
        while busy_signal.is_busy() {}
    }

    /// Checks if device is still busy - use a timeout since we don't have a busy pin on the inky-frame
    // pub(crate) fn is_busy(&mut self, timeout: u8) -> bool {
    //     delay.delay_ms(timeout);
    //     return true;
    // }

    pub(crate) fn reset(&mut self, busy_signal: &mut impl IsBusy) {
        let _ = self.rst.set_low();
        // delay.delay_ms(10);
        let _ = self.rst.set_high();
        // delay.delay_ms(10);
        self.wait_until_idle(busy_signal);
    }
}
