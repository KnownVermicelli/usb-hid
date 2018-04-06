# `cortex-m-quickstart`

> A template for building applications for ARM Cortex-M microcontrollers

# [Documentation](https://docs.rs/cortex-m-quickstart)

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Developing tools

You will need:
- something to flash stm32. I'm using stm32flash: 	https://git.code.sf.net/p/stm32flash/code
- linux tools for building stm32 stuff. You can find instruction on japaric blog in "rust your arm microcontroller".

Notes:
- PA11 (USBDM) and PA12 (USBDP) are usb pins.
- https://github.com/ah-/anne-key seems not to set any usb pins, so it's using defaults. should check how code in stm32f103xx crate works.