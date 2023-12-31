use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};

/// When trying to parse u8 to one of the color types
#[derive(Debug, PartialEq, Eq)]
pub struct OutOfColorRangeParseError(u8);
impl core::fmt::Display for OutOfColorRangeParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Outside of possible Color Range: {}", self.0)
    }
}
impl OutOfColorRangeParseError {
    fn _new(size: u8) -> OutOfColorRangeParseError {
        OutOfColorRangeParseError(size)
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OctColor {
    /// Black Color
    Black = 0x00,
    /// White Color
    White = 0x01,
    /// Green Color
    Green = 0x02,
    /// Blue Color
    Blue = 0x03,
    /// Red Color
    Red = 0x04,
    /// Yellow Color
    Yellow = 0x05,
    /// Orange Color
    Orange = 0x06,
    /// HiZ / Clean Color
    HiZ = 0x07,
}

impl From<()> for OctColor {
    fn from(_: ()) -> OctColor {
        OctColor::White
    }
}

impl From<BinaryColor> for OctColor {
    fn from(b: BinaryColor) -> OctColor {
        match b {
            BinaryColor::On => OctColor::Black,
            BinaryColor::Off => OctColor::White,
        }
    }
}

impl From<OctColor> for embedded_graphics::pixelcolor::Rgb888 {
    fn from(b: OctColor) -> Self {
        let (r, g, b) = b.rgb();
        Self::new(r, g, b)
    }
}

impl From<embedded_graphics::pixelcolor::Rgb565> for OctColor {
    fn from(p: embedded_graphics::pixelcolor::Rgb565) -> OctColor {
        use embedded_graphics::prelude::RgbColor;
        let colors = [
            OctColor::Black,
            OctColor::White,
            OctColor::Green,
            OctColor::Blue,
            OctColor::Red,
            OctColor::Yellow,
            OctColor::Orange,
            OctColor::HiZ,
        ];
        // if the user has already mapped to the right color space, it will just be in the list
        if let Some(found) = colors.iter().find(|c| c.rgb() == (p.r(), p.g(), p.b())) {
            return *found;
        }

        // This is not ideal but just pick the nearest color
        *colors
            .iter()
            .map(|c| (c, c.rgb()))
            .map(|(c, (r, g, b))| {
                let dist = (i32::from(r) - i32::from(p.r())).pow(2)
                    + (i32::from(g) - i32::from(p.g())).pow(2)
                    + (i32::from(b) - i32::from(p.b())).pow(2);
                (c, dist)
            })
            .min_by_key(|(_c, dist)| *dist)
            .map(|(c, _)| c)
            .unwrap_or(&OctColor::White)
    }
}

impl From<embedded_graphics::pixelcolor::Rgb555> for OctColor {
    fn from(p: embedded_graphics::pixelcolor::Rgb555) -> OctColor {
        use embedded_graphics::prelude::RgbColor;
        let colors = [
            OctColor::Black,
            OctColor::White,
            OctColor::Green,
            OctColor::Blue,
            OctColor::Red,
            OctColor::Yellow,
            OctColor::Orange,
            OctColor::HiZ,
        ];
        // if the user has already mapped to the right color space, it will just be in the list
        if let Some(found) = colors.iter().find(|c| c.rgb() == (p.r(), p.g(), p.b())) {
            return *found;
        }

        // This is not ideal but just pick the nearest color
        *colors
            .iter()
            .map(|c| (c, c.rgb()))
            .map(|(c, (r, g, b))| {
                let dist = (i32::from(r) - i32::from(p.r())).pow(2)
                    + (i32::from(g) - i32::from(p.g())).pow(2)
                    + (i32::from(b) - i32::from(p.b())).pow(2);
                (c, dist)
            })
            .min_by_key(|(_c, dist)| *dist)
            .map(|(c, _)| c)
            .unwrap_or(&OctColor::White)
    }
}

impl From<embedded_graphics::pixelcolor::Rgb888> for OctColor {
    fn from(p: embedded_graphics::pixelcolor::Rgb888) -> OctColor {
        use embedded_graphics::prelude::RgbColor;
        let colors = [
            OctColor::Black,
            OctColor::White,
            OctColor::Green,
            OctColor::Blue,
            OctColor::Red,
            OctColor::Yellow,
            OctColor::Orange,
            OctColor::HiZ,
        ];
        // if the user has already mapped to the right color space, it will just be in the list
        if let Some(found) = colors.iter().find(|c| c.rgb() == (p.r(), p.g(), p.b())) {
            return *found;
        }

        // This is not ideal but just pick the nearest color
        *colors
            .iter()
            .map(|c| (c, c.rgb()))
            .map(|(c, (r, g, b))| {
                let dist = (i32::from(r) - i32::from(p.r())).pow(2)
                    + (i32::from(g) - i32::from(p.g())).pow(2)
                    + (i32::from(b) - i32::from(p.b())).pow(2);
                (c, dist)
            })
            .min_by_key(|(_c, dist)| *dist)
            .map(|(c, _)| c)
            .unwrap_or(&OctColor::White)
    }
}

impl From<embedded_graphics::pixelcolor::raw::RawU4> for OctColor {
    fn from(b: embedded_graphics::pixelcolor::raw::RawU4) -> Self {
        use embedded_graphics::prelude::RawData;
        OctColor::from_nibble(b.into_inner()).unwrap()
    }
}

impl PixelColor for OctColor {
    type Raw = embedded_graphics::pixelcolor::raw::RawU4;
}

impl OctColor {
    /// Gets the Nibble representation of the Color as needed by the display
    pub fn get_nibble(self) -> u8 {
        self as u8
    }
    /// Converts two colors into a single byte for the Display
    pub fn colors_byte(a: OctColor, b: OctColor) -> u8 {
        a.get_nibble() << 4 | b.get_nibble()
    }

    ///Take the nibble (lower 4 bits) and convert to an OctColor if possible
    pub fn from_nibble(nibble: u8) -> Result<OctColor, OutOfColorRangeParseError> {
        match nibble & 0xf {
            0x00 => Ok(OctColor::Black),
            0x01 => Ok(OctColor::White),
            0x02 => Ok(OctColor::Green),
            0x03 => Ok(OctColor::Blue),
            0x04 => Ok(OctColor::Red),
            0x05 => Ok(OctColor::Yellow),
            0x06 => Ok(OctColor::Orange),
            0x07 => Ok(OctColor::HiZ),
            e => Err(OutOfColorRangeParseError(e)),
        }
    }
    ///Split the nibbles of a single byte and convert both to an OctColor if possible
    pub fn split_byte(byte: u8) -> Result<(OctColor, OctColor), OutOfColorRangeParseError> {
        let low = OctColor::from_nibble(byte & 0xf)?;
        let high = OctColor::from_nibble((byte >> 4) & 0xf)?;
        Ok((high, low))
    }
    /// Converts to limited range of RGB values.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            OctColor::White => (0xff, 0xff, 0xff),
            OctColor::Black => (0x00, 0x00, 0x00),
            OctColor::Green => (0x00, 0xff, 0x00),
            OctColor::Blue => (0x00, 0x00, 0xff),
            OctColor::Red => (0xff, 0x00, 0x00),
            OctColor::Yellow => (0xff, 0xff, 0x00),
            OctColor::Orange => (0xff, 0x80, 0x00),
            OctColor::HiZ => (0x80, 0x80, 0x80), /* looks greyish */
        }
    }
}
