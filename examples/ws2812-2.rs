#![no_main]
#![no_std]

use stm32l4xx_hal as hal;
use ws2812_spi as ws2812;
#[macro_use]
extern crate cortex_m_rt as rt;
use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use crate::rt::entry;
use crate::rt::ExceptionFrame;
use crate::ws2812::Ws2812;
use cortex_m::peripheral::Peripherals;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        // Constrain clocking registers
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
        let clocks = rcc // full speed (64 & 80MHz) use the 16MHZ HSI osc + PLL (but slower / intermediate values need MSI)
            .cfgr
            .sysclk(80.mhz())
            .pclk1(80.mhz())
            .pclk2(80.mhz())
            .freeze(&mut flash.acr, &mut pwr);

        let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        let (sck, miso, mosi) = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
                gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
                gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
            )
        });

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        let mut ws = Ws2812::new(spi);

        const NUM_LEDS: usize = 8;
        let mut data = [RGB8::default(); NUM_LEDS];
        enum Mode {
            Rainbow,
            WhiteInOut,
            Flash,
            Kitt,
        }
        match Mode::Kitt {
            Mode::Rainbow => loop {
                for j in 0..(256 * 5) {
                    for i in 0..NUM_LEDS {
                        data[i] =
                            wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                    }
                    ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                    // ws.write(data.iter().cloned()).unwrap();
                    delay.delay_ms(5u8);
                }
            },
            Mode::WhiteInOut => loop {
                for j in ((0..256).chain((0..256).rev())).cycle() {
                    let data = [RGB8::new(j as u8, j as u8, j as u8); NUM_LEDS];

                    // ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                    ws.write(data.iter().cloned()).unwrap();
                    //delay.delay_ms(5u8);
                }
            },
            Mode::Flash => loop {
                let r = 0..NUM_LEDS;

                for j in r.clone().chain(r.rev()).cycle() {
                    let col1 = RGB8::new(255, 200, 160);
                    let col2 = RGB8::new(0, 0, 0);
                    data.iter_mut().enumerate().for_each(|(i, v)| {
                        if i == j {
                            *v = col1
                        } else {
                            *v = col2
                        }
                    });

                    // ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                    ws.write(data.iter().cloned()).unwrap();

                    if j == 0 || j == 255 {
                        delay.delay_ms(255u8);
                        // delay.delay_ms(255u8);
                    }
                    delay.delay_ms(16u8);
                }
            },
            Mode::Kitt => {
                let up = 0..NUM_LEDS;
                let down = (0..NUM_LEDS).rev();
                let pause = core::iter::repeat(8).take(8);
                let pause_short = core::iter::repeat(8).take(2);
                // let pause_short = core::iter::once(8);

                // let mut seq = down.chain(pause_short).chain(up).chain(pause).cycle();
                let mut seq = up.chain(pause_short).chain(down).chain(pause).cycle();
                let mut prev = seq.next().unwrap();
                let mut c = 0;
                const RAMPDOWN: u8 = 5;
                loop {
                    data.iter_mut().for_each(|v| {
                        if v.r < RAMPDOWN {
                            v.r = 0;
                        } else {
                            v.r -= RAMPDOWN;
                        }
                    });
                    delay.delay_ms(8u8);
                    if c == 10 {
                        let s = seq.next().unwrap();

                        // full brightness lags behind one frame (simulate turn on time of 80s lightbulbs)
                        if prev < NUM_LEDS {
                            data[prev] = RGB8::new(255, 0, 0);
                        }
                        if s < NUM_LEDS {
                            data[s] = RGB8::new(128, 0, 0);
                        }
                        prev = s;
                        c = 0;
                    }
                    c += 1;
                    // ws.write(data.iter().cloned()).unwrap();
                    ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                }
            }
        }
    }
    loop {
        continue;
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
