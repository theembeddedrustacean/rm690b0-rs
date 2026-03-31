use crate::{ColorMode, ControllerInterface, DrawTarget, ResetInterface, Rm690b0Driver};
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
        let bytes_per_pixel = self.color_mode.bytes_per_pixel();

        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < self.config.width as i32
                && coord.y >= 0
                && coord.y < self.config.height as i32
            {
                let x = coord.x as u32;
                let y = coord.y as u32;
                let index = ((y * self.config.width as u32 + x) as usize) * bytes_per_pixel;
                let pixel_end = index + bytes_per_pixel;

                if pixel_end <= self.framebuffer.len() {
                    // Convert from generic color into Rgb888
                    let rgb: Rgb888 = color.into();

                    match self.color_mode {
                        ColorMode::Rgb888 => {
                            self.framebuffer[index] = rgb.r();
                            self.framebuffer[index + 1] = rgb.g();
                            self.framebuffer[index + 2] = rgb.b();
                        }
                        ColorMode::Rgb666 => {
                            // Store RGB666 left-aligned in each byte (6 MSBs used).
                            self.framebuffer[index] = rgb.r() & 0xFC;
                            self.framebuffer[index + 1] = rgb.g() & 0xFC;
                            self.framebuffer[index + 2] = rgb.b() & 0xFC;
                        }
                        ColorMode::Rgb565 => {
                            let r5 = (rgb.r() >> 3) as u16;
                            let g6 = (rgb.g() >> 2) as u16;
                            let b5 = (rgb.b() >> 3) as u16;
                            let packed = (r5 << 11) | (g6 << 5) | b5;

                            self.framebuffer[index] = (packed >> 8) as u8;
                            self.framebuffer[index + 1] = (packed & 0xFF) as u8;
                        }
                        ColorMode::Gray8 => {
                            // Integer luma approximation: 0.299R + 0.587G + 0.114B.
                            let gray = ((rgb.r() as u16 * 77)
                                + (rgb.g() as u16 * 150)
                                + (rgb.b() as u16 * 29))
                                >> 8;
                            self.framebuffer[index] = gray as u8;
                        }
                    }
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
