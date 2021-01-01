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
const matrix_map: [i16; 21 * 19] = [
    -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, -1, -1, -1, -1, -1, -1, 8, 9, 10, 11,
    12, 13, 14, 15, 16, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, -1, -1, -1, -1, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, -1, -1, 53, 54,
    55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, -1, -1, -1, -1, 69, 70, 71, 72, 73, 74,
    75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, -1, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, -1, -1, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, -1, -1, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132,
    133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
    152, 153, -1, -1, -1, -1, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167,
    168, 169, 170, -1, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185,
    186, 187, -1, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203,
    204, -1, -1, -1, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219,
    220, 221, -1, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237,
    238, -1, -1, -1, -1, -1, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252,
    253, -1, -1, -1, 254, 255, 256, 257, 258, 259, 260, 261, 262, 263, 264, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 265, 266, 267, 268, 269, 270, 271, 272, 273, 274, -1, -1, -1, -1,
    -1, 275, 276, 277, 278, 279, 280, 281, 282, 283, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 283, 284, 285, 286, 287, 288, 289, 290, 291, -1, -1, -1,
];
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

        const NUM_LEDS: usize = 154;
        let mut data = [RGB8::default(); NUM_LEDS];
        enum Mode {
            Rainbow,
            WhiteInOut,
            Flash,
            Kitt,
        }
        for mode in [Mode::Rainbow, Mode::WhiteInOut, Mode::Flash, Mode::Kitt]
            .iter()
            .cycle()
        {
            match mode {
                Mode::Rainbow => {
                    for _ in 0..1 {
                        for j in 0..(256 * 5) {
                            for i in 0..NUM_LEDS {
                                data[i] = wheel(
                                    (((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8,
                                );
                            }
                            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                            // ws.write(data.iter().cloned()).unwrap();
                            delay.delay_ms(5u8);
                        }
                    }
                }
                Mode::WhiteInOut => {
                    for _ in 0..1 {
                        for j in ((0..256).chain((0..256).rev())) {
                            let data = [RGB8::new(j as u8, j as u8, j as u8); NUM_LEDS];

                            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                            // ws.write(data.iter().cloned()).unwrap();
                            //delay.delay_ms(5u8);
                        }
                    }
                }
                Mode::Flash => {
                    for _ in 0..1 {
                        let r = 0..NUM_LEDS;

                        for j in r.clone().chain(r.rev()) {
                            let col1 = RGB8::new(255, 200, 160);
                            let col2 = RGB8::new(0, 0, 0);
                            data.iter_mut().enumerate().for_each(|(i, v)| {
                                if i == j {
                                    *v = col1
                                } else {
                                    *v = col2
                                }
                            });

                            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                            // ws.write(data.iter().cloned()).unwrap();

                            if j == 0 || j == 255 {
                                delay.delay_ms(255u8);
                                // delay.delay_ms(255u8);
                            }
                            delay.delay_ms(16u8);
                        }
                    }
                }
                Mode::Kitt => {
                    for _ in 0..2 {
                        let up = 0..NUM_LEDS;
                        let down = (0..NUM_LEDS).rev();
                        let pause = core::iter::repeat(8).take(8);
                        let pause_short = core::iter::repeat(8).take(2);
                        // let pause_short = core::iter::once(8);

                        // let mut seq = down.chain(pause_short).chain(up).chain(pause).cycle();
                        let mut seq = up.chain(pause_short).chain(down).chain(pause);
                        let mut prev = seq.next().unwrap();
                        let mut c = 0;
                        const RAMPDOWN: u8 = 64;
                        for cur in seq {
                            data.iter_mut().for_each(|v| {
                                if v.r < RAMPDOWN {
                                    v.r = 0;
                                } else {
                                    v.r -= RAMPDOWN;
                                }
                            });
                            delay.delay_ms(8u8);
                            if c == 1 {
                                // let s = seq.next().unwrap();

                                // full brightness lags behind one frame (simulate turn on time of 80s lightbulbs)
                                if prev < NUM_LEDS {
                                    data[prev] = RGB8::new(255, 0, 0);
                                }
                                if cur < NUM_LEDS {
                                    data[cur] = RGB8::new(128, 0, 0);
                                }
                                prev = cur;
                                c = 0;
                            }
                            c += 1;
                            // ws.write(data.iter().cloned()).unwrap();
                            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                        }
                    }
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
