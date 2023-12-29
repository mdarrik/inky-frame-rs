use embedded_graphics::prelude::*;

use super::{color::OctColor, DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};

/// Full size buffer for use with the Inky Frame's Display
/// Handles making inky frame compatible with Embedded Graphics
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 2 * HEIGHT]`
pub struct InkyFrameDisplay {
    buffer: [u8; WIDTH as usize / 2 * HEIGHT as usize],
    rotation: DisplayRotation,
}

impl Default for InkyFrameDisplay {
    fn default() -> Self {
        InkyFrameDisplay {
            buffer: [OctColor::colors_byte(DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_COLOR);
                WIDTH as usize / 2 * HEIGHT as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for InkyFrameDisplay {
    type Color = OctColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for InkyFrameDisplay {
    fn size(&self) -> Size {
        Size::new(WIDTH.into(), HEIGHT.into())
    }
}

impl InkyFrameDisplay {
    /// Clears the buffer of the display with the chosen background color
    pub fn clear_buffer(&mut self, background_color: OctColor) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = OctColor::colors_byte(background_color, background_color);
        }
    }

    /// Returns the buffer
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Sets the rotation of the display
    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    /// Helperfunction for the Embedded Graphics draw trait
    fn draw_helper(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<OctColor>,
    ) -> Result<(), core::convert::Infallible> {
        let rotation = self.rotation();
        let buffer = self.get_mut_buffer();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, upper) =
            find_oct_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        let (mask, color_nibble) = if upper {
            (0x0f, color.get_nibble() << 4)
        } else {
            (0xf0, color.get_nibble())
        };

        match buffer.get_mut(index) {
            None => {
                #[cfg(feature = "defmt")]
                defmt::warn!(
                    "index out of buffer, {} - point ({}, {})",
                    index,
                    point.x,
                    point.y
                );
                ()
            }
            Some(i) => {
                *i = (*i & mask) | color_nibble;
            }
        }
        Ok(())
    }
}

/// Displayrotation
#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

impl Default for DisplayRotation {
    fn default() -> Self {
        DisplayRotation::Rotate0
    }
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait OctDisplay: DrawTarget<Color = OctColor> {
    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;
}

// Checks if a pos is outside the defined display
fn outside_display(p: Point, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    if p.x < 0 || p.y < 0 {
        return true;
    }
    let (x, y) = (p.x as u32, p.y as u32);
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            }
        }
    }
    false
}

//returns index position in the u8-slice and the bit-position inside that u8
fn find_oct_position(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    rotation: DisplayRotation,
) -> (u32, bool) {
    let (new_x, new_y) = find_rotation(x, y, width, height, rotation);
    (
        new_x / 2 + (width / 2) * new_y,
        // is this an upper or lower bit position in the slice
        (new_x & 0x1) == 0,
    )
}

fn find_rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u32) {
    let new_x;
    let new_y;
    match rotation {
        DisplayRotation::Rotate0 => {
            new_x = x;
            new_y = y;
        }
        DisplayRotation::Rotate90 => {
            new_x = width - 1 - y;
            new_y = x;
        }
        DisplayRotation::Rotate180 => {
            new_x = width - 1 - x;
            new_y = height - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            new_x = y;
            new_y = height - 1 - x;
        }
    }
    (new_x, new_y)
}
