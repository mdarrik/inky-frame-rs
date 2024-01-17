use crate::display::traits::Command;

use embedded_hal::{digital::OutputPin, spi::SpiDevice};

use super::IsBusy;
/// Interface for the display
pub(crate) struct DisplayInterface<SPI, DC, RST> {
    /// SPI
    spi: SPI,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Resetting
    rst: RST,
}

impl<SPI, DC, RST> DisplayInterface<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(dc: DC, spi: SPI, rst: RST) -> Self {
        DisplayInterface { spi, dc, rst }
    }

    /// Basic function for sending [Commands](Command).
    ///
    /// Enables direct interaction with the device with the help of [data()](DisplayInterface::data())
    pub(crate) fn cmd<T: Command>(&mut self, command: T) -> Result<(), SPI::Error> {
        // low for commands
        let _ = self.dc.set_low();

        // Transfer the command over spi
        self.write(&[command.address()])
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](Epd4in2::command())
    pub(crate) fn data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();

        for val in data.iter().copied() {
            // Transfer data one u8 at a time over spi
            self.write(&[val])?;
        }

        Ok(())
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    pub(crate) fn cmd_with_data<T: Command>(
        &mut self,
        command: T,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.cmd(command)?;
        self.data(data)
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    pub(crate) fn data_x_times(&mut self, val: u8, repetitions: u32) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(&[val])?;
        }
        Ok(())
    }

    /// spi write helper/abstraction function
    fn write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        // transfer spi data
        self.spi.write(data)?;
        Ok(())
    }

    /// waits until the device is not busy
    pub(crate) fn wait_until_idle(&mut self, busy_signal: &mut impl IsBusy) {
        while busy_signal.is_busy() {}
    }

    /// reset the display using the reset pin
    pub(crate) fn reset(&mut self, busy_signal: &mut impl IsBusy) {
        let _ = self.rst.set_low();
        let _ = self.rst.set_high();
        self.wait_until_idle(busy_signal);
    }
}
