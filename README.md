Collection of examples for the [STM32F4VE dev board](https://stm32-base.org/boards/STM32F407VET6-STM32-F4VE-V2.0)

These examples were setup from the crates documentation and from:
* https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/analog-stopwatch-with-spi-ssd1306.rs
* https://jonathanklimt.de/electrics/programming/rust-STM32F103-blink/
* https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/analog-stopwatch-with-spi-ssd1306.rs
* https://wapl.es/electronics/rust/2018/04/30/ssd1306-driver.html

To run any of the examples:
* openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg
* cargo run --release --bin <binary_name>

This will include debugging and logs from the semihosting crate.

Check this link for more documentation: https://rust-embedded.github.io/book/start/semihosting.html
