use crate::{ControllerInterface, DrawTarget, ResetInterface, Rm690b0Driver};
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_graphics_core::prelude::*;

impl<IFACE, RST, C> DrawTarget for Rm690b0Driver<IFACE, RST, C>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
    C: PixelColor + Into<Rgb888>,
{
    type Color = C;
    // Drawing to the framebuffer in memory is infallible.
    // Errors happen during flush with SPI comms.
    type Error = core::convert::Infallible;

    /// Draws a single pixel to the internal framebuffer.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < self.config.width as i32
                && coord.y >= 0
                && coord.y < self.config.height as i32
            {
                let x = coord.x as u32;
                let y = coord.y as u32;
                let index = ((y * self.config.width as u32 + x) * 3) as usize;

                if index + 2 < self.framebuffer.len() {
                    // Convert from generic color into Rgb888
                    let rgb: Rgb888 = color.into();

                    self.framebuffer[index] = rgb.r();
                    self.framebuffer[index + 1] = rgb.g();
                    self.framebuffer[index + 2] = rgb.b();
                }
            }
        }
        Ok(())
    }
}

impl<IFACE, RST, C> OriginDimensions for Rm690b0Driver<IFACE, RST, C>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
    C: PixelColor,
{
    fn size(&self) -> Size {
        Size::new((self.config.width) as u32, (self.config.height) as u32)
    }
}
