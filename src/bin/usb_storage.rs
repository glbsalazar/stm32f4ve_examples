//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: any STM32F4 with a OTG FS peripheral and a 25MHz HSE crystal
#![no_std]
#![no_main]

use panic_halt as _;

use core::cell::RefCell;
use cortex_m::{interrupt::Mutex, peripheral::NVIC};
use cortex_m_rt::entry;
use stm32f4xx_hal::otg_fs::{UsbBus, UsbBusType, USB};
use stm32f4xx_hal::sdio;
use stm32f4xx_hal::stm32::interrupt;
use stm32f4xx_hal::{prelude::*, stm32};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_scsi::{BlockDevice, BlockDeviceError, Scsi};

static mut EP_MEMORY: [u32; 1024] = [0; 1024];
static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static USB_DEV: Mutex<RefCell<Option<UsbDevice<UsbBusType>>>> = Mutex::new(RefCell::new(None));
static USB_STORAGE: Mutex<RefCell<Option<usbd_scsi::Scsi<UsbBusType, Storage>>>> =
    Mutex::new(RefCell::new(None));

struct Storage {
    host: RefCell<stm32f4xx_hal::sdio::Sdio>,
}

impl BlockDevice for Storage {
    const BLOCK_BYTES: usize = 512;

    fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        let sdio = &mut self.host.borrow_mut();

        // Use this until const generics and try_into is a thing
        let mut b = [0; 512];

        sdio.read_block(lba, &mut b)
            .map_err(|_| BlockDeviceError::HardwareError)?;

        block.copy_from_slice(&b);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        let sdio = &mut self.host.borrow_mut();

        // Use this until const generics and try_into is a thing
        let mut b = [0; 512];
        b.copy_from_slice(&block);

        sdio.write_block(lba, &b)
            .map_err(|_| BlockDeviceError::WriteError)?;

        Ok(())
    }

    fn max_lba(&self) -> u32 {
        let sdio = &self.host.borrow();

        sdio.card().map(|c| c.block_count() - 1).unwrap_or(0)
    }
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .require_pll48clk()
        .freeze();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();

    // SDIO
    let clk = gpioc.pc12.into_alternate_af12();
    let cmd = gpiod.pd2.into_alternate_af12();
    let dat0 = gpioc.pc8.into_alternate_af12();
    let dat1 = gpioc.pc9.into_alternate_af12();
    let dat2 = gpioc.pc10.into_alternate_af12();
    let dat3 = gpioc.pc11.into_alternate_af12();

    let mut sd = sdio::Sdio::new(dp.SDIO, (clk, cmd, dat0, dat1, dat2, dat3), clocks);
    sd.init_card(sdio::ClockFreq::F4Mhz).unwrap();

    let sdhc = Storage {
        host: RefCell::new(sd),
    };

    // unsafe {
    let usb = USB {
        usb_global: dp.OTG_FS_GLOBAL,
        usb_device: dp.OTG_FS_DEVICE,
        usb_pwrclk: dp.OTG_FS_PWRCLK,
        pin_dm: gpioa.pa11.into_alternate_af10(),
        pin_dp: gpioa.pa12.into_alternate_af10(),
        hclk: clocks.hclk(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
    unsafe {
        USB_BUS = Some(usb_bus);
    }
    let scsi = Scsi::new(
        unsafe { USB_BUS.as_ref().unwrap() },
        64,
        sdhc,
        "Fake Co.",
        "Test",
        "F407",
    );
    // }
    let usb_dev = UsbDeviceBuilder::new(
        unsafe { USB_BUS.as_ref().unwrap() },
        UsbVidPid(0x16c0, 0x27dd),
    )
    .manufacturer("Fake company")
    .product("SDUSB")
    .serial_number("TEST")
    .device_class(usbd_mass_storage::USB_CLASS_MSC)
    .build();

    cortex_m::interrupt::free(|cs| {
        USB_DEV.borrow(cs).replace(Some(usb_dev));
        USB_STORAGE.borrow(cs).replace(Some(scsi));
    });
    // };

    unsafe {
        NVIC::unmask(stm32::Interrupt::OTG_FS);
    }

    loop {
        continue;
    }
}

#[interrupt]
fn OTG_FS() {
    cortex_m::interrupt::free(|cs| {
        let mut dev = USB_DEV.borrow(cs).borrow_mut();
        let usb_dev = dev.as_mut().unwrap();

        let mut scsi = USB_STORAGE.borrow(cs).borrow_mut();
        let scsi = scsi.as_mut().unwrap();

        if !usb_dev.poll(&mut [scsi]) {
            return;
        }
    });
}
