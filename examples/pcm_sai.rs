//! Blinks an LED

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#![no_main]
#![allow(unused_imports)]

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

use crate::sh::hio;

use core::fmt::Write;
use hal::i2c::I2c;
use hal::stm32::sai1;
use hal::stm32::SAI1;

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

    // led.set_high();
    // timer.delay_ms(1000 as u32);
    // led.set_low();
    // timer.delay_ms(1000 as u32);
    // led.set_high();
    // timer.delay_ms(1000 as u32);
    let mut pcm5122 = Pcm5122::new(i2c);
    led.set_high().unwrap();
    init_default(&mut pcm5122, &mut timer).unwrap();
    led.set_low().unwrap();
    let mut _lrclk_in = gpiob
        .pb9
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af13(&mut gpiob.moder, &mut gpiob.afrh);

    let mut _bclk_in = gpiob
        .pb10
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af13(&mut gpiob.moder, &mut gpiob.afrh);

    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut _data_out = gpioc
        .pc3
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper)
        .into_af13(&mut gpioc.moder, &mut gpioc.afrl);

    // let sai1 = dp.SAI1;
    // let cha = sai1.cha;

    // setup CR1
    dp.SAI1.cha.cr1.write(|w| {
        w.lsbfirst()
            .msb_first() // big endian
            .ds()
            .bit16() // DS = 16bit
            .mode()
            .slave_tx() // slave tx
    });

    // setup CR2
    dp.SAI1.cha.cr2.write(
        |w| w.fth().quarter2(), // threshold half
    );
    // setup frcr
    dp.SAI1.cha.frcr.write(|w| unsafe {
        w
            //.fspol()            .rising_edge() // FS is active high
            .fsdef()
            .set_bit() // FS is start of frame and channel indication
            .fsall()
            .bits(15) // FS high for half frame
            .frl()
            .bits(31) // frame is 32bits
    });

    // setup slotr
    dp.SAI1.cha.slotr.write(|w| unsafe {
        w.sloten()
            .bits(0b11) // enable slots 0, 1
            .nbslot()
            .bits(1) // two slots
            .slotsz()
            .bit16() // 16bit per slot
    });

    led.set_high().unwrap();
    timer.delay_ms(100u32);
    // for i in 0..1 {
    //     dp.SAI1.cha.dr.write(|w| unsafe { w.data().bits(i as u32) })
    // }

    if dp.SAI1.cha.sr.read().wckcfg().is_wrong() {
        panic!("bad wckcfg");
    }
    if dp.SAI1.cha.sr.read().ovrudr().is_overrun() {
        panic!("overrun");
    }

    // dp.SAI1
    //     .cha
    //     .dr
    //     .write(|w| unsafe { w.data().bits(0b1010101010101010) });
    while dp.SAI1.cha.sr.read().flvl().is_empty() {
        // for _ in 0..8 {
        led.set_high().unwrap();
        dp.SAI1
            .cha
            .dr
            .write(|w| unsafe { w.data().bits(0b1010101010101010) });
        led.set_low().unwrap();
    }

    // while dp.SAI1.cha.cr2.read().fth().is_empty() {
    //     dp.SAI1
    //         .cha
    //         .dr
    //         .write(|w| unsafe { w.data().bits(0b1010101010101010) });
    // }

    dp.SAI1.cha.cr1.write(|w| w.saien().enabled());
    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
