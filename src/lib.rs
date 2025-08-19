//! # RM690B0 Driver Crate
//!
//! An embedded-graphics compatible driver for the RM690B0 display controller IC.
//!
//! This driver is not embedded-hal compatible, but provides a generic interface
//! for controlling the RM690B0 display controller.
//! Different displays can be supported by implementing the `ControllerInterface` and `ResetInterface` traits.
//! This is because the RM690B0 is used in displays with various interfaces such as SPI or QSPI.
//! Additionally, the reset pin is controlled via GPIO or I2C GPIO expander.
//!
//! The driver currently supports the Lilygo T4-S3 AMOLED display out of the box, but can be extended to support other displays.
//!
//! ## Usage
//! 1. Implement the `ControllerInterface` trait for the controller driving interface Ex. QSPI
//! 2. Implement the `ResetInterface` trait for the Reset pin.
//! 3. Create a `Rm690b0Driver` instance with the display interface and reset pin.
//! 4. Use the driver to draw using `embedded-graphics`.
//!
//! If you are going to use heap allocated framebuffer, you will need to make sure that an allocator is available in your environment.
//! In some crates this is done by enabling the `alloc` feature.
//!
//! ## Feature Flags
#![doc = document_features::document_features!()]
//!
//! ## Examples
//! See the `examples` directory for a usage example with the Lilygo T4-S3 Display.
//!
//! The Lilygo T4-S3 Display controls the RM690B0 via an ESP32-S3 over QSPI and uses GPIO output to control the reset pin.
//! The example implementation uses a PSRAM heap allocated framebuffer and DMA for efficient transfers.
//!
//! The schematic is available here: <https://github.com/Xinyuan-LilyGO/LilyGo-AMOLED-Series/blob/master/schematic/T4-S3-240719.pdf>
//!
//! To run the example, with the Lilygo T4-S3 Display, clone the project and run following command from the project root:
//! ```bash
//! cargo run --example lilygo_t4_s3 --features "lilygo_t4_s3 --release"
//! ```
//!

#![no_std]
#[cfg(feature = "lilygo_t4_s3")]
pub mod displays;

#[cfg(feature = "lilygo_t4_s3")]
pub use displays::lilygo_t4_s3::*;

extern crate alloc;

mod graphics_core;

use alloc::boxed::Box;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_hal::delay::DelayNs;

/// Configuration for the display dimensions.
#[derive(Debug, Clone, Copy)]
pub struct DisplaySize {
    /// Display width in pixels.
    pub width: u16,
    /// Display height in pixels.
    pub height: u16,
}

impl DisplaySize {
    pub const fn new(width: u16, height: u16) -> Self {
        DisplaySize { width, height }
    }
}

/// RM690B0 Driver Errors
#[derive(Debug)]
pub enum DriverError<InterfaceError, ResetError> {
    /// Error originating from the display interface (QSPI/SPI/I2C).
    InterfaceError(InterfaceError),
    /// Error originating from the reset pin control.
    ResetError(ResetError),
    /// Invalid configuration provided to the driver.
    InvalidConfiguration(&'static str),
}

/// Trait to implement the controller communication interface (QSPI, SPI, etc.).
pub trait ControllerInterface {
    /// The specific error type for this interface implementation.
    type Error;

    /// Sends a command byte to the display.
    fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error>;

    /// Sends data bytes to the display following a command.
    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), Self::Error>;

    /// Sends pixel data
    fn send_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error>;
}

/// Trait for controlling the hardware reset pin.
pub trait ResetInterface {
    /// The specific error type for this reset implementation.
    type Error;

    /// Performs the hardware reset sequence according to the datasheet definition.
    fn reset(&mut self) -> Result<(), Self::Error>;
}

/// RM690B0 Command Set
pub mod commands {
    pub const NOP: u8 = 0x00;
    pub const SWRESET: u8 = 0x01;
    pub const RDDID: u8 = 0x04; // Read Display Identification Information
    pub const RDNUMED: u8 = 0x05; // Read Number of Errors on DSI
    pub const RDDPM: u8 = 0x0A; // Read Display Power Mode
    pub const RDDMADCTR: u8 = 0x0B; // Read Display MADCTR
    pub const RDDCOLMOD: u8 = 0x0C; // Read Display Pixel Format
    pub const RDDIM: u8 = 0x0D; // Read Display Image Mode
    pub const RDDSM: u8 = 0x0E; // Read Display Signal Mode
    pub const RDDSDR: u8 = 0x0F; // Read Display Self-Diagnostic Result
    pub const SLPIN: u8 = 0x10;
    pub const SLPOUT: u8 = 0x11;
    pub const PTLON: u8 = 0x12; // Partial Display Mode On
    pub const NORON: u8 = 0x13; // Normal Display Mode On
    pub const INVOFF: u8 = 0x20; // Display Inversion Off
    pub const INVON: u8 = 0x21; // Display Inversion On
    pub const ALLPOFF: u8 = 0x22; // All Pixel Off
    pub const ALLPON: u8 = 0x23; // All Pixel On
    pub const DISPOFF: u8 = 0x28;
    pub const DISPON: u8 = 0x29;
    pub const CASET: u8 = 0x2A; // Column Address Set
    pub const RASET: u8 = 0x2B; // Row Address Set
    pub const RAMWR: u8 = 0x2C; // Memory Write
    pub const PTLAR: u8 = 0x30; // Partial Area
    pub const TEOFF: u8 = 0x34; // Tearing Effect Off
    pub const TEON: u8 = 0x35; // Tearing Effect On
    pub const MADCTR: u8 = 0x36; // Memory Data Access Control
    pub const IDMOFF: u8 = 0x38; // Idle Mode Off
    pub const IDMON: u8 = 0x39; // Idle Mode On
    pub const COLMOD: u8 = 0x3A; // Interface Pixel Format
    pub const RAMWRC: u8 = 0x3C; // Memory Continuous Write
    pub const STESL: u8 = 0x44; // Set Tear Scan Line
    pub const GSL: u8 = 0x45; // Get Scan Line
    pub const DSTBON: u8 = 0x4F; // Deep Standby Mode On
    pub const WRDISBV: u8 = 0x51; // Write Display Brightness
    pub const RDDISBV: u8 = 0x52; // Read Display Brightness
    pub const WRCTRLD: u8 = 0x53; // Write CTRL Display
    pub const RDCTRLD: u8 = 0x54; // Read CTRL Display
    pub const WRRADACL: u8 = 0x55; // RAD_ACL Control
    pub const COLORTEMP: u8 = 0x55; // Color Temperature Selection (shared with WRRADACL)
    pub const WRHBM: u8 = 0x63; // Write HBM Display Brightness
    pub const RDHBM: u8 = 0x64; // Read HBM Display Brightness
    pub const HBM_MODE: u8 = 0x66; // Set HBM Mode
    pub const FR_LEVEL: u8 = 0x67; // Frame Rate Level Control
    pub const COLSET: u8 = 0x70; // Interface Pixel Format Set
    pub const COLOPT: u8 = 0x80; // Interface Pixel Format Option
    pub const RDDDBS: u8 = 0xA1; // Read DDB Start
    pub const RDDDBC: u8 = 0xA8; // Read DDB Continuous
    pub const RDFCS: u8 = 0xAA; // Read First Checksum
    pub const RDCCS: u8 = 0xAF; // Read Continue Checksum
}

/// Color modes supported by the RM690B0 display controller.
pub enum ColorMode {
    /// 16-bit RGB565 format
    Rgb565,
    /// 24-bit RGB888 format
    Rgb888,
    /// 18-bit RGB666 format
    Rgb666,
    /// 8-bit 256 Gray
    Gray8,
}

impl ColorMode {
    /// Returns the number of bytes per pixel for the color format.
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            ColorMode::Rgb565 => 2,
            ColorMode::Rgb888 => 3,
            ColorMode::Rgb666 => 3,
            ColorMode::Gray8 => 1,
        }
    }
}

/// Computes the framebuffer size (in bytes) for a given display and color mode.
pub const fn framebuffer_size(display: DisplaySize, color: ColorMode) -> usize {
    (display.width as usize) * (display.height as usize) * color.bytes_per_pixel()
}

/// Framebuffer enum to hold either a static array or a boxed array
pub enum Framebuffer {
    Static(&'static mut [u8]),
    Heap(Box<[u8]>),
}

impl Framebuffer {
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Framebuffer::Static(ref mut arr) => arr,
            Framebuffer::Heap(ref mut boxed) => boxed,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Framebuffer::Static(ref arr) => arr,
            Framebuffer::Heap(ref boxed) => boxed,
        }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl core::ops::Deref for Framebuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            Framebuffer::Static(arr) => arr,
            Framebuffer::Heap(boxed) => boxed,
        }
    }
}

impl core::ops::DerefMut for Framebuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Framebuffer::Static(arr) => arr,
            Framebuffer::Heap(boxed) => boxed,
        }
    }
}

/// Main Driver for the RM690B0 display controller.
pub struct Rm690b0Driver<IFACE, RST>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
{
    interface: IFACE,
    reset: RST,
    framebuffer: Framebuffer,
    config: DisplaySize,
}

impl<IFACE, RST> Rm690b0Driver<IFACE, RST>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
{
    /// Creates a new driver instance with static array and initializes the display.
    pub fn new_static<DELAY, const N: usize>(
        interface: IFACE,
        reset: RST,
        color: ColorMode,
        config: DisplaySize,
        mut delay: DELAY,
        framebuffer: &'static mut [u8; N],
    ) -> Result<Self, DriverError<IFACE::Error, RST::Error>>
    where
        DELAY: DelayNs,
    {
        let mut driver = Self {
            interface,
            reset,
            framebuffer: Framebuffer::Static(&mut framebuffer[..]),
            config,
        };
        driver.hard_reset()?;
        driver.initialize_display(&mut delay, color)?;
        Ok(driver)
    }

    /// Creates a new driver instance with a boxed array framebuffer.
    pub fn new_heap<DELAY, const N: usize>(
        interface: IFACE,
        reset: RST,
        color: ColorMode,
        config: DisplaySize,
        mut delay: DELAY,
    ) -> Result<Self, DriverError<IFACE::Error, RST::Error>>
    where
        DELAY: DelayNs,
    {
        let mut driver = Self {
            interface,
            reset,
            framebuffer: Framebuffer::Heap(Box::new([0u8; N])),
            config,
        };
        driver.hard_reset()?;
        driver.initialize_display(&mut delay, color)?;
        Ok(driver)
    }

    /// Performs a hardware reset using the provided `ResetPin` implementation.
    pub fn hard_reset(&mut self) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.reset.reset().map_err(DriverError::ResetError)?;
        Ok(())
    }

    /// Sends the essential initialization command sequence to the display.
    pub fn initialize_display<DELAY>(
        &mut self,
        delay: &mut DELAY,
        color: ColorMode,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>>
    where
        DELAY: DelayNs,
    {
        self.send_command(commands::SLPOUT)?;
        delay.delay_ms(120);

        // Manufacturer-Specific Initialization
        self.send_command_with_data(0xFE, &[0x20])?;
        self.send_command_with_data(0x26, &[0x0A])?;
        self.send_command_with_data(0x24, &[0x80])?;
        self.send_command_with_data(0x5A, &[0x51])?;
        self.send_command_with_data(0x5B, &[0x2E])?;
        self.send_command_with_data(0xFE, &[0x00])?;

        // Sets Interface Pixel Format to 24-bit/pixel (RGB888)
        match color {
            ColorMode::Rgb565 => {
                // Set pixel format to RGB565
                self.send_command_with_data(commands::COLMOD, &[0x55])?;
            }
            ColorMode::Rgb888 => {
                // Set pixel format to RGB888
                self.send_command_with_data(commands::COLMOD, &[0x77])?;
            }
            ColorMode::Rgb666 => {
                // Set pixel format to RGB666
                self.send_command_with_data(commands::COLMOD, &[0x66])?;
            }
            ColorMode::Gray8 => {
                // Set pixel format to 8-bit grayscale
                self.send_command_with_data(commands::COLMOD, &[0x11])?;
            }
        }

        self.send_command_with_data(commands::TEON, &[0x00])?;

        self.send_command(commands::DISPON)?;
        delay.delay_ms(20);

        // Display Brightness Set to Maximum
        self.send_command_with_data(commands::WRDISBV, &[0xFF])?;

        Ok(())
    }

    /// Send a command with no data
    fn send_command(&mut self, cmd: u8) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.interface
            .send_command(cmd)
            .map_err(DriverError::InterfaceError)
    }

    /// Helper to send a command with associated data parameters
    fn send_command_with_data(
        &mut self,
        cmd: u8,
        data: &[u8],
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.interface
            .send_command_with_data(cmd, data)
            .map_err(DriverError::InterfaceError)?;
        Ok(())
    }

    /// Sleep Mode In (SLPIN)
    pub fn sleep_in<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>>
    where
        DELAY: DelayNs,
    {
        self.send_command(commands::SLPIN)?;
        delay.delay_ms(5);
        Ok(())
    }

    /// Sleep Out (SLPOUT)
    pub fn sleep_out<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>>
    where
        DELAY: DelayNs,
    {
        self.send_command(commands::SLPOUT)?;
        delay.delay_ms(5);
        Ok(())
    }

    /// Turns the display panel off
    pub fn display_off(&mut self) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.send_command(commands::DISPOFF)
    }

    /// Turns the display panel on
    pub fn display_on(&mut self) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.send_command(commands::DISPON)
    }

    /// Sets the active drawing window on the display RAM.
    pub fn set_window(
        &mut self,
        x_start: u16,
        y_start: u16,
        x_end: u16,
        y_end: u16,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        if x_end < x_start || y_end < y_start || x_end >= 480 || y_end >= self.config.height {
            return Err(DriverError::InvalidConfiguration(
                "Invalid window dimensions",
            ));
        }

        // CASET
        self.send_command_with_data(
            commands::CASET,
            &[
                (x_start >> 8) as u8,
                (x_start & 0xFF) as u8,
                (x_end >> 8) as u8,
                (x_end & 0xFF) as u8,
            ],
        )?;

        // RASET
        self.send_command_with_data(
            commands::RASET,
            &[
                (y_start >> 8) as u8,
                (y_start & 0xFF) as u8,
                (y_end >> 8) as u8,
                (y_end & 0xFF) as u8,
            ],
        )?;
        Ok(())
    }

    /// Sets the Memory Data Access Control (MADCTR) register.
    pub fn set_madctr(&mut self, value: u8) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.send_command_with_data(commands::MADCTR, &[value])
    }

    /// Sets the display brightness (0x00 - 0xFF for RM690B0).
    pub fn set_brightness(
        &mut self,
        value: u8,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.send_command_with_data(commands::WRDISBV, &[value])
    }

    /// Writes the contents of the framebuffer to the display RAM.
    pub fn flush(&mut self) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        // Set window to full display
        self.set_window(0, 0, self.config.width - 1, self.config.height - 1)?;
        self.interface
            .send_pixels(&self.framebuffer)
            .map_err(DriverError::InterfaceError)?;
        Ok(())
    }

    pub fn partial_flush(
        &mut self,
        x_start: u16,
        x_end: u16,
        y_start: u16,
        y_end: u16,
        color: ColorMode,
    ) -> Result<(), DriverError<IFACE::Error, RST::Error>> {
        self.set_window(x_start, y_start, x_end, y_end)?;
        let bytes_per_pixel = color.bytes_per_pixel();
        let fb_width = self.config.width as usize * bytes_per_pixel;
        let width = (x_end - x_start + 1) as usize;
        let height = (y_end - y_start + 1) as usize;
        let mut pixel_data = alloc::vec::Vec::with_capacity(width * height * bytes_per_pixel);

        for y in 0..height {
            let offset = (y_start as usize + y) * fb_width + (x_start as usize * bytes_per_pixel);
            let row_end = offset + (width * bytes_per_pixel);
            if offset < self.framebuffer.len() && row_end <= self.framebuffer.len() {
                pixel_data.extend_from_slice(&self.framebuffer[offset..row_end]);
            } else {
                return Err(DriverError::InvalidConfiguration(
                    "Framebuffer slice out of bounds",
                ));
            }
        }

        self.interface
            .send_pixels(&pixel_data)
            .map_err(DriverError::InterfaceError)?;
        Ok(())
    }
}
