// Example setup from
// https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/analog-stopwatch-with-spi-ssd1306.rs
// https://jonathanklimt.de/electrics/programming/rust-STM32F103-blink/
// stm32_rust project in the pc

#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use hal::{
    prelude::*,
    stm32,
};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // important really important
        // https://stackoverflow.com/questions/56179131/cannot-receive-interrupt-on-pe0-stm32
        dp.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Configure the USART
        let gpioa = dp.GPIOA.split();
        let (tx, rx) = (
            gpioa.pa9.into_alternate_af7(),
            gpioa.pa10.into_alternate_af7(),
        );
        let usart_config = stm32f4xx_hal::serial::config::Config::default();
        let usart_config = usart_config.baudrate(115_200.bps());
        let mut usart1 =
            hal::serial::Serial::usart1(dp.USART1, (tx, rx), usart_config, clocks).unwrap();
        let buf = "Hello World \r\n".as_bytes();
        let mut write_offset = 0;
        let count = buf.len();
        while write_offset < count {
            match usart1.write(buf[write_offset]) {
                Ok(_) => {
                    write_offset += 1;
                }
                _ => {}
            }
        }

        loop {}
    }

    loop {}
}
