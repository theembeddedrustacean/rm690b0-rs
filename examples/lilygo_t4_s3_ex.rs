#![no_std]
#![no_main]

use embedded_hal::digital::OutputPin;
use rm690b0_rs::{
    framebuffer_size,
    ColorMode,
    DisplaySize,
    Lgt4s3Driver,
    ResetDriver,
    Rm690b0Driver,
    DMA_CHUNK_SIZE, // Note: Changed to Rm690b0Driver
};

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyle,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};

extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
    delay::Delay,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Io, Level, Output, OutputConfig},
    i2c::master::{Config as I2cConfig, I2c},
    main,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::Rate,
};
use esp_println::println;

esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let delay = Delay::new();

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // --- SPI Configuration ---
    // Try lowering the frequency to 10 MHz initially
    let lcd_spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40_u32)) // <-- LOWERED CLOCK SPEED
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sio0(peripherals.GPIO14)
    .with_sio1(peripherals.GPIO10)
    .with_sio2(peripherals.GPIO16)
    .with_sio3(peripherals.GPIO12)
    .with_cs(peripherals.GPIO11)
    .with_sck(peripherals.GPIO15)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    // --- Pin Configuration ---
    let output = Output::new(peripherals.GPIO13, Level::High, OutputConfig::default());
    let mut pwr_en = Output::new(peripherals.GPIO9, Level::High, OutputConfig::default());

    let reset = ResetDriver::new(output);
    let ws_driver = Lgt4s3Driver::new(lcd_spi);

    // --- Display Setup ---
    const DISPLAY_SIZE: DisplaySize = DisplaySize::new(450, 600);
    const FB_SIZE: usize = framebuffer_size(DISPLAY_SIZE, ColorMode::Rgb888);

    println!("Initializing RM690B0 Display...");
    let mut display = Rm690b0Driver::new_heap::<_, FB_SIZE>(
        ws_driver,
        reset,
        ColorMode::Rgb888,
        DISPLAY_SIZE,
        delay,
    )
    .expect("Display initialization failed");

    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

    let text_style = TextStyleBuilder::new().alignment(Alignment::Center).build();

    let text = "Hello, LilyGo T4-S3 Display!";

    loop {
        for col in (0..=DISPLAY_SIZE.width as i32).step_by(10) {
            // Draw the text at the new position
            Text::with_text_style(text, Point::new(col, 300), character_style, text_style)
                .draw(&mut display)
                .unwrap();

            // Flush the framebuffer to the display
            if let Err(e) = display.flush() {
                println!("Error flushing display: {:?}", e);
            }

            // Clear the buffer for the next frame
            display.clear(Rgb888::BLACK).unwrap();
        }
    }
}
