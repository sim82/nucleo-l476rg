//! Blinks an LED

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;
use crate::hal::delay::Delay;
use crate::hal::rcc::{PllConfig, PllDivider};
// #[macro_use(block)]
// extern crate nb;

use crate::hal::prelude::*;
use crate::rt::entry;
use crate::rt::ExceptionFrame;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain(); // .constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    // Try a different clock configuration
    let clock_cfg = 4;
    let clocks = match clock_cfg {
        0 => rcc.cfgr.freeze(&mut flash.acr, &mut pwr), // directly use 16MHz HSI without PLL
        1 => {
            rcc // full speed (64 & 80MHz) use the 16MHZ HSI osc + PLL (but slower / intermediate values need MSI)
                .cfgr
                .sysclk(80.mhz())
                .pclk1(80.mhz())
                .pclk2(80.mhz())
                .freeze(&mut flash.acr, &mut pwr)
        }
        2 => {
            rcc // weird clock frequencies can be achieved by using the internal multispeed oscillator (MSI) + PLL
                .cfgr
                .msi(stm32l4xx_hal::rcc::MsiFreq::RANGE4M)
                .sysclk(37.mhz())
                .pclk1(37.mhz())
                .pclk2(37.mhz())
                .freeze(&mut flash.acr, &mut pwr)
        }
        3 => rcc // HSI48 does not become ready => does not work
            .cfgr
            .hsi48(true)
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr, &mut pwr),
        4 => {
            rcc // run at 8MHz with explicit pll config (otherwise rcc auto config fall back to 16MHz HSI)
                .cfgr
                .msi(stm32l4xx_hal::rcc::MsiFreq::RANGE8M)
                .sysclk_with_pll(
                    8.mhz(),
                    PllConfig::new(
                        0b001,            // / 2
                        0b1000,           // * 8
                        PllDivider::Div8, // /8
                    ),
                )
                .pclk1(8.mhz())
                .pclk2(8.mhz())
                .freeze(&mut flash.acr, &mut pwr)
        }
        _ => panic!("unhandled clock config"),
    };

    let mut timer = Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut led = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    loop {
        // note: delay_ms does not automatically compensate for systick wraparound. E.g. at 80MHz the 24bit systick timer wraps around every 209ms.
        led.set_high().unwrap();
        timer.delay_ms(209u16);
        led.set_low().unwrap();
        timer.delay_ms(50u16);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
