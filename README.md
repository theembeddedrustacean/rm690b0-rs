# RM690B0 Driver Crate

An embedded-graphics compatible driver for the RM690B0 display controller IC.

This driver is not embedded-hal compatible but provides a generic interface for controlling the RM690B0 display controller. Different displays can be supported by implementing the `ControllerInterface` and `ResetInterface` traits. This is because the RM690B0 is used in different displays with various controller interfaces such as SPI or QSPI. Additionally, the reset pin is controlled via GPIO or an I2C GPIO expander.

The driver currently incorporates support for the LilyGo T4-S3 display out of the box but can be extended to support other displays using the RM690B0 controller.

## Usage

1. Implement the `ControllerInterface` trait for the controller driving interface (e.g., QSPI).
2. Implement the `ResetInterface` trait for the Reset pin.
3. Create a `Rm690b0Driver` instance with the display interface and reset pin.
4. Use the driver to draw using `embedded-graphics`.

If you are going to use a heap-allocated framebuffer, you will need to ensure that an allocator is available in your environment. In some crates, this is done by enabling the `alloc` feature.

## Examples

See the `examples` directory for a usage example with the LilyGo T4-S3 Display.

The LilyGo T4-S3 Display controls the RM690B0 via an ESP32-S3 over QSPI and GPIO output for the reset pin. The example implementation uses a PSRAM heap-allocated framebuffer and DMA for efficient transfers.

The schematic is available here: https://github.com/Xinyuan-LilyGo/LilyGo-AMOLED-Series/blob/master/schematic/T4-S3-240719.pdf

To run the example, with the LilyGo T4-S3 Display, clone the project and run following command from the project root:

```bash
cargo run --example LilyGo_t4_s3_ex --features "LilyGo_t4_s3" --release
```