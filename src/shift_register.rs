use crate::display::IsBusy;
use embedded_hal::digital::v2::{InputPin, OutputPin};
pub struct InkyFrameShiftRegister<GpioOutput, GpioInput> {
    clock_pin: GpioOutput,
    latch_pin: GpioOutput,
    out_pin: GpioInput,
}

const IS_BUSY_FLAG: u8 = 7;

impl<GpioOutput, GpioInput, GpioE> InkyFrameShiftRegister<GpioOutput, GpioInput>
where
    GpioOutput: OutputPin<Error = GpioE>,
    GpioInput: InputPin<Error = GpioE>,
{
    pub fn new(clock_pin: GpioOutput, latch_pin: GpioOutput, out_pin: GpioInput) -> Self {
        InkyFrameShiftRegister {
            clock_pin,
            latch_pin,
            out_pin,
        }
    }

    pub fn read_register(&mut self) -> Result<u8, GpioE> {
        self.latch_pin.set_low()?;
        self.latch_pin.set_high()?;
        let mut result = 0u8;
        let mut bits = 8u8;

        while bits > 0 {
            bits -= 1;
            result <<= 1;
            if self.out_pin.is_high()? {
                result |= 1;
            } else {
                result |= 0;
            }
            self.clock_pin.set_low()?;
            self.clock_pin.set_high()?;
        }

        Ok(result)
    }

    pub fn read_register_bit(&mut self, bit_index: u8) -> Result<u8, GpioE> {
        Ok(self.read_register()? & (1u8 << bit_index))
    }
}

impl<GpioOutput, GpioInput, GpioE> IsBusy for InkyFrameShiftRegister<GpioOutput, GpioInput>
where
    GpioOutput: OutputPin<Error = GpioE>,
    GpioInput: InputPin<Error = GpioE>,
{
    fn is_busy(&mut self) -> bool {
        if let Ok(res) = self.read_register_bit(IS_BUSY_FLAG) {
            return res == 0;
        } else {
            return false;
        }
    }
}
