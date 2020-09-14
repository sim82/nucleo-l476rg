//! Blinks an LED

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;
// #[macro_use(block)]
// extern crate nb;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::rt::entry;
use crate::rt::ExceptionFrame;

use hal::i2c::I2c;

use nucleo_l476rg::pcm5122::init_default;
use nucleo_l476rg::pcm5122::Pcm5122;

#[entry]
fn main() -> ! {
    //let mut hstdout = hio::hstdout().unwrap();

    //writeln!(hstdout, "Hello, world!").unwrap();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain(); // .constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    // Try a different clock configuration
    //let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);
    // let clocks = rcc
    //     .cfgr
    //     .hclk(48.mhz())
    //     .sysclk(80.mhz())
    //     .pclk1(24.mhz())
    //     .pclk2(24.mhz())
    //     .freeze(&mut flash.acr, &mut pwr);
    // let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.afrh);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut led = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let mut timer = Delay::new(cp.SYST, clocks);

    // let mut pa9 = gpioa
    //     .pa10
    //     .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut scl = gpiob
        .pb6
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let mut sda = gpiob
        .pb7
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    sda.internal_pull_up(&mut gpiob.pupdr, true);
    let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1r1);
    let mut pcm5122 = Pcm5122::new(i2c);

    led.set_high().unwrap();
    timer.delay_ms(1000 as u32);
    led.set_low().unwrap();
    timer.delay_ms(1000 as u32);
    led.set_high().unwrap();
    timer.delay_ms(1000 as u32);
    init_default(&mut pcm5122, &mut timer).unwrap();

    led.set_low().unwrap();
    let lrclk_in = gpiob
        .pb9
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr);

    let bclk_in = gpiob
        .pb10
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut data_out = gpioc
        .pc3
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    let mut data = 0u16;
    loop {
        for inc in 200..400 {
            let dtmp = data;
            data = data.wrapping_add(inc);
            while lrclk_in.is_low().unwrap() {}
            for i in 0..16 {
                while bclk_in.is_low().unwrap() {}

                if dtmp & (0b1000000000000000 >> i) != 0 {
                    data_out.set_high().unwrap();
                } else {
                    data_out.set_low().unwrap();
                }

                while bclk_in.is_high().unwrap() {}
            }
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
