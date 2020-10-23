// Example setup from
// https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/analog-stopwatch-with-spi-ssd1306.rs
// https://jonathanklimt.de/electrics/programming/rust-STM32F103-blink/
// https://wapl.es/electronics/rust/2018/04/30/ssd1306-driver.html
// stm32_rust project in the pc
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{
    prelude::*,
    stm32,
};

// Drawing stuff
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle, Triangle},
    style::PrimitiveStyleBuilder,
};
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
// end drawing stuff

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

        // I2C
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb6.into_alternate_af4_open_drain();
        let sda = gpiob.pb7.into_alternate_af4_open_drain();
        let i2c = hal::i2c::I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks);

        // Draw something from example
        let interface = I2CDIBuilder::new().init(i2c);
        let mut disp: GraphicsMode<_, _> = Builder::new()
            .size(DisplaySize128x32)
            .connect(interface)
            .into();
        disp.init().unwrap();

        let size = 10;
        let offset = Point::new(10, (42 / 2) - (size / 2) - 1);
        let spacing = size + 10;

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        // screen outline
        // default display size is 128x64 if you don't pass a _DisplaySize_
        // enum to the _Builder_ struct
        Rectangle::new(Point::new(0, 0), Point::new(128, 31))
            .into_styled(style)
            .draw(&mut disp)
            .unwrap();

        // Triangle
        Triangle::new(
            Point::new(0, size),
            Point::new(size / 2, 0),
            Point::new(size, size),
        )
        .translate(offset)
        .into_styled(style)
        .draw(&mut disp)
        .unwrap();

        // Move over to next position
        let offset = offset + Point::new(spacing, 0);

        // Draw a square
        Rectangle::new(Point::new(0, 0), Point::new(size, size))
            .translate(offset)
            .into_styled(style)
            .draw(&mut disp)
            .unwrap();

        // Move over a bit more
        let offset = offset + Point::new(spacing, 0);

        // Circle
        Circle::new(Point::new(size / 2, size / 2), size as u32 / 2)
            .translate(offset)
            .into_styled(style)
            .draw(&mut disp)
            .unwrap();

        disp.flush().unwrap();
        // Finish draw something from example

        loop {}
    }

    loop {}
}
