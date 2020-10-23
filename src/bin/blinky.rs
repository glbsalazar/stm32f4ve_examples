// Example setup from
// https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/analog-stopwatch-with-spi-ssd1306.rs
// https://jonathanklimt.de/electrics/programming/rust-STM32F103-blink/
// stm32_rust project in the pc

// TODO:
// 2 buttons, 1 timer, 2 leds
// SPI? UART?
// I2C -> oled screen

#![no_main]
#![no_std]

// Halt on panic
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{
    gpio::{Edge, ExtiPin, Input, PullUp},
    prelude::*,
    stm32,
};
use hal::stm32::interrupt;

// Needed to use the gpio with the interrupt
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};

static GPIOPA6: Mutex<
    RefCell<Option<hal::gpio::gpioa::PA6<hal::gpio::Output<hal::gpio::PushPull>>>>,
> = Mutex::new(RefCell::new(None));

static GPIOPA7: Mutex<
    RefCell<Option<hal::gpio::gpioa::PA7<hal::gpio::Output<hal::gpio::PushPull>>>>,
> = Mutex::new(RefCell::new(None));

static BUTTON0: Mutex<RefCell<Option<hal::gpio::gpioe::PE4<Input<PullUp>>>>> =
    Mutex::new(RefCell::new(None));

static BUTTON1: Mutex<RefCell<Option<hal::gpio::gpioe::PE3<Input<PullUp>>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(_cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // important really important
        // https://stackoverflow.com/questions/56179131/cannot-receive-interrupt-on-pe0-stm32
        dp.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Set up the LEDs PA6 and PA7
        let gpioa = dp.GPIOA.split();
        let gpioe = dp.GPIOE.split();


        let led0 = gpioa.pa6.into_push_pull_output();
        let led1 = gpioa.pa7.into_push_pull_output();
        let (mut but0, mut but1) = (
            gpioe.pe4.into_pull_up_input(),
            gpioe.pe3.into_pull_up_input(),
        );

        // Set the led within the mutex
        cortex_m::interrupt::free(move |cs| {
            GPIOPA6.borrow(cs).replace(Some(led0));
            GPIOPA7.borrow(cs).replace(Some(led1));
        });

        but0.make_interrupt_source(&mut dp.SYSCFG);
        but0.enable_interrupt(&mut dp.EXTI);
        but0.trigger_on_edge(&mut dp.EXTI, Edge::FALLING);
        but1.make_interrupt_source(&mut dp.SYSCFG);
        but1.enable_interrupt(&mut dp.EXTI);
        but1.trigger_on_edge(&mut dp.EXTI, Edge::FALLING);

        cortex_m::interrupt::free(|cs| BUTTON0.borrow(cs).replace(Some(but0)));
        cortex_m::interrupt::free(|cs| BUTTON1.borrow(cs).replace(Some(but1)));

        // Create a delay abstraction based on SysTick
        // let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

        // Enable interrupts
        stm32::NVIC::unpend(hal::stm32::Interrupt::EXTI4);
        stm32::NVIC::unpend(hal::stm32::Interrupt::EXTI3);
        unsafe {
            stm32::NVIC::unmask(hal::stm32::Interrupt::EXTI3);
            stm32::NVIC::unmask(hal::stm32::Interrupt::EXTI4);
        };

        loop {}
    }

    loop {}
}

// uC specific interrupts are defined as interrupts
#[interrupt]
fn EXTI3() {
    free(|cs| {
        let mut btn_ref = BUTTON1.borrow(cs).borrow_mut();
        let mut led_ref = GPIOPA7.borrow(cs).borrow_mut();
        if let (Some(ref mut btn), Some(ref mut led)) = (btn_ref.deref_mut(), led_ref.deref_mut()) {
            // We cheat and don't bother checking _which_ exact interrupt line fired - there's only
            // ever going to be one in this example.
            btn.clear_interrupt_pending_bit();
            let _ = led.toggle();
        }
    });
}

#[interrupt]
fn EXTI4() {
    free(|cs| {
        let mut btn_ref = BUTTON0.borrow(cs).borrow_mut();
        let mut led_ref = GPIOPA6.borrow(cs).borrow_mut();
        if let (Some(ref mut btn), Some(ref mut led)) = (btn_ref.deref_mut(), led_ref.deref_mut()) {
            // We cheat and don't bother checking _which_ exact interrupt line fired - there's only
            // ever going to be one in this example.
            btn.clear_interrupt_pending_bit();
            let _ = led.toggle();
        }
    });
}
