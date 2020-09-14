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

use crate::sh::hio;
use core::fmt::Write;
use hal::i2c::I2c;

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
    let mut pa6 = gpioa
        .pa6
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

    let mut i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1r1);

    let mut read_buf: [u8; 16] = [0; 16];
    let dac_addr = 0x9a >> 1;

    led.set_high();
    timer.delay_ms(1000 as u32);
    led.set_low();
    timer.delay_ms(1000 as u32);
    led.set_high();
    timer.delay_ms(1000 as u32);
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.677766200,0,0x9A,0x80,Write,ACK page 0
                                                           // 1.678086200,1,0x9B,0x00,Read,NAK ?
    i2c.write_read(dac_addr, &[0x81, 0x11], &mut read_buf[..1]); // 1.679802350,3,0x9A,0x81,Write,ACK reset RSTM + RSTR
                                                                 // 1.679946350,3,0x9A,0x11,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.681006250,5,0x9A,0x80,Write,ACK page 0 1.681326250,6,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x81, 0x00], &mut read_buf[..1]); // 1.681707800,8,0x9A,0x81,Write,ACK reset -> 0
                                                                 // 1.681851800,8,0x9A,0x00,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.682285000,10,0x9A,0x80,Write,ACK page 0 1.682605000,11,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x82, 0x10], &mut read_buf[..1]); // 1.683045700,13,0x9A,0x82,Write,ACK
                                                                 // 1.683189700,13,0x9A,0x10,Write,ACK standby mode on, powerdown off
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.683590700,15,0x9A,0x80,Write,ACK 1.683910700,16,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x82, 0x11], &mut read_buf[..1]); // 1.684302450,18,0x9A,0x82,Write,ACK
                                                                 // 1.684446450,18,0x9A,0x11,Write,ACK standby mode on, powerdown on
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.685635650,20,0x9A,0x80,Write,ACK 1.685955650,21,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x88], &mut read_buf[..1]); // 1.686347300,23,0x9A,0x88,Write,ACK gpio? 1.686667300,24,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.687042100,26,0x9A,0x80,Write,ACK 1.687362100,27,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x88, 0x24], &mut read_buf[..1]); // 1.687804250,29,0x9A,0x88,Write,ACK
                                                                 // 1.687948250,29,0x9A,0x24,Write,ACK gpio: 3 + 6 output, qrest input GPIO 3 & 6 == xtal enable
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.688328350,31,0x9A,0x80,Write,ACK 1.688648350,32,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd2], &mut read_buf[..1]); // 1.689036050,34,0x9A,0xD2,Write,ACK gpio 3 output selection ? 1.689356050,35,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.689753450,37,0x9A,0x80,Write,ACK 1.690073450,38,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd2, 0x02], &mut read_buf[..1]); // 1.690474750,40,0x9A,0xD2,Write,ACK gpio 3 output selection Register GPIO3 output
                                                                 // 1.690618750,40,0x9A,0x02,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.690987350,42,0x9A,0x80,Write,ACK 1.691307350,43,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd5], &mut read_buf[..1]); // 1.691768650,45,0x9A,0xD5,Write,ACK gpio 6? 1.692088650,46,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.692476300,48,0x9A,0x80,Write,ACK 1.692796300,49,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd5, 0x02], &mut read_buf[..1]); // 1.693157450,51,0x9A,0xD5,Write,ACK Register GPIO6 output
                                                                 // 1.693301450,51,0x9A,0x02,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.693666550,53,0x9A,0x80,Write,ACK 1.693986550,54,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd6, 0x10], &mut read_buf[..1]); // 1.694342700,56,0x9A,0xD6,Write,ACK
                                                                 // 1.694662700,57,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.695019900,59,0x9A,0x80,Write,ACK 1.695339900,60,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd6, 0x20], &mut read_buf[..1]); // 1.695695300,62,0x9A,0xD6,Write,ACK
                                                                 // 1.695839300,62,0x9A,0x20,Write,ACK GPIP6 ouput high
    timer.delay_ms(100u16);
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.723057700,64,0x9A,0x80,Write,ACK 1.723377650,65,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xde, 0x10], &mut read_buf[..1]); // 1.723735800,67,0x9A,0xDE,Write,ACK clock detector state read
                                                                 // 1.724055800,68,0x9B,0x2F,Read,NAK ? pll unlocked, clocks missing (no shit)
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.724437500,70,0x9A,0x80,Write,ACK 1.724757500,71,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xde, 0xd6, 0x00], &mut read_buf[..1]); // 1.725111400,73,0x9A,0xD6,Write,ACK GPIO ouput control, all low
                                                                       // 1.725255400,73,0x9A,0x00,Write,ACK
    timer.delay_ms(20u16);
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.753192650,75,0x9A,0x80,Write,ACK 1.753512650,76,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xde], &mut read_buf[..1]); // 1.753874150,78,0x9A,0xDE,Write,ACK clock detector state read 1.754194150,79,0x9B,0x6F,Read,NAK even more missing
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.754581550,81,0x9A,0x80,Write,ACK 1.754901550,82,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd6, 0x04], &mut read_buf[..1]); // 1.755291600,84,0x9A,0xD6,Write,ACK GPIO 3 high
                                                                 // 1.755435600,84,0x9A,0x04,Write,ACK
    timer.delay_ms(30u16);
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.783074150,86,0x9A,0x80,Write,ACK 1.783394150,87,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xde], &mut read_buf[..1]); // 1.783750550,89,0x9A,0xDE,Write,ACK read clock state, clocks missing 1.784070550,90,0x9B,0x2F,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.784481050,92,0x9A,0x80,Write,ACK 1.784801050,93,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x89, 0x11], &mut read_buf[..1]); // 1.785154850,95,0x9A,0x89,Write,ACK BCK & LRCK master (yay!)
                                                                 // 1.785298850,95,0x9A,0x11,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.785681600,97,0x9A,0x80,Write,ACK 1.786001550,98,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x8c, 0x7f], &mut read_buf[..1]); // 1.786352450,100,0x9A,0x8C,Write,ACK BCK & LRCK divider functional
                                                                 // 1.786496450,100,0x9A,0x7F,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 1.786849700,102,0x9A,0x80,Write,ACK 1.787169700,103,0x9B,0x00,Read,NAK BCK & LRCK start
    i2c.write_read(dac_addr, &[0xa1, 0x3f], &mut read_buf[..1]); // 1.787518300,105,0x9A,0xA1,Write,ACK master mode LRCK divider
                                                                 // 1.787662250,105,0x9A,0x3F,Write,ACK divide by 64
                                                                 // LRCK 4MHz -> 384KHz
                                                                 // }

    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  1.788014800,107,0x9A,0x80,Write,ACK  1.788334800,108,0x9B,0x00,Read,NAK
                                                           //  1.788684300,110,0x9A,0x88,Write,ACK
                                                           //  1.788828300,110,0x9A,0x2C,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  1.789182750,112,0x9A,0x80,Write,ACK
                                                           //  1.789502750,113,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd3], &mut read_buf[..1]); //  1.789867500,115,0x9A,0xD3,Write,ACK
                                                           //  1.790187500,116,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  1.790541950,118,0x9A,0x80,Write,ACK
                                                           //  1.790861950,119,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd3, 0x02], &mut read_buf[..1]); //  1.791311400,121,0x9A,0xD3,Write,ACK
                                                                 //  1.791455400,121,0x9A,0x02,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  1.791811200,123,0x9A,0x80,Write,ACK
                                                           //  1.792131200,124,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd6, 0x0c], &mut read_buf[..1]); //  1.792480100,126,0x9A,0xD6,Write,ACK
                                                                 //  1.792624100,126,0x9A,0x0C,Write,ACK

    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  16.467275650,128,0x9A,0x80,Write,ACK
                                                           //  16.467595650,129,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x82, 0x10], &mut read_buf[..1]); //  16.467988200,131,0x9A,0x82,Write,ACK
                                                                 //  16.468132200,131,0x9A,0x10,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  16.474144800,133,0x9A,0x80,Write,ACK
                                                           //  16.474464800,134,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xd6, 0x28], &mut read_buf[..1]); //  16.474832550,136,0x9A,0xD6,Write,ACK
                                                                 //  16.474976550,136,0x9A,0x28,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  16.475350450,138,0x9A,0x80,Write,ACK
                                                           //  16.475670450,139,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa8], &mut read_buf[..1]); //  16.476030250,141,0x9A,0xA8,Write,ACK
                                                           //  16.476350250,142,0x9B,0x02,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); //  16.476707850,144,0x9A,0x80,Write,ACK
                                                           //  16.477027850,145,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa8], &mut read_buf[..1]); //  16.477386700,147,0x9A,0xA8,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.475350450,138,0x9A,0x80,Write,ACK
                                                           // 16.475670450,139,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa8], &mut read_buf[..1]); // 16.476030250,141,0x9A,0xA8,Write,ACK
                                                           // 16.476350250,142,0x9B,0x02,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.476707850,144,0x9A,0x80,Write,ACK
                                                           // 16.477027850,145,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa8, 0x00], &mut read_buf[..1]); // 16.477386700,147,0x9A,0xA8,Write,ACK
                                                                 // 16.477530700,147,0x9A,0x00,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.482812350,149,0x9A,0x80,Write,ACK
                                                           // 16.483132350,150,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa5, 0x7b], &mut read_buf[..1]); // 16.488012650,152,0x9A,0xA5,Write,ACK
                                                                 // 16.488156650,152,0x9A,0x7B,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.488527400,154,0x9A,0x80,Write,ACK
                                                           // 16.488847400,155,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x84], &mut read_buf[..1]); // 16.489209450,157,0x9A,0x84,Write,ACK
                                                           // 16.489529450,158,0x9B,0x01,Read,NAK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.489894550,160,0x9A,0x80,Write,ACK
                                                           // 16.490214550,161,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x84, 0x00], &mut read_buf[..1]); // 16.490572350,163,0x9A,0x84,Write,ACK
                                                                 // 16.490716350,163,0x9A,0x00,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.491087600,165,0x9A,0x80,Write,ACK
                                                           // 16.491407600,166,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x8e, 0x30], &mut read_buf[..1]); // 16.491763600,168,0x9A,0x8E,Write,ACK
                                                                 // 16.491907600,168,0x9A,0x30,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.492270300,170,0x9A,0x80,Write,ACK
                                                           // 16.492590300,171,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x9b, 0x00], &mut read_buf[..1]); // 16.492994350,173,0x9A,0x9B,Write,ACK
                                                                 // 16.493138350,173,0x9A,0x00,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.493502800,175,0x9A,0x80,Write,ACK
                                                           // 16.493822800,176,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x9c, 0x03], &mut read_buf[..1]); // 16.494202750,178,0x9A,0x9C,Write,ACK
                                                                 // 16.494346750,178,0x9A,0x03,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.494736400,180,0x9A,0x80,Write,ACK
                                                           // 16.495056400,181,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x9d, 0x03], &mut read_buf[..1]); // 16.495420750,183,0x9A,0x9D,Write,ACK
                                                                 // 16.495564750,183,0x9A,0x03,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.495943000,185,0x9A,0x80,Write,ACK
                                                           // 16.496263000,186,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x9e, 0x07], &mut read_buf[..1]); // 16.496618550,188,0x9A,0x9E,Write,ACK
                                                                 // 16.496762550,188,0x9A,0x07,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.497132100,190,0x9A,0x80,Write,ACK
                                                           // 16.497452100,191,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa0, 0x0f], &mut read_buf[..1]); // 16.497811650,193,0x9A,0xA0,Write,ACK
                                                                 // 16.497955650,193,0x9A,0x0F,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.498321050,195,0x9A,0x80,Write,ACK
                                                           // 16.498641050,196,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa1, 0x1f], &mut read_buf[..1]); // 16.498994800,198,0x9A,0xA1,Write,ACK
                                                                 // 16.499138800,198,0x9A,0x1F,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.499495150,200,0x9A,0x80,Write,ACK
                                                           // 16.499815150,201,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa3, 0x02], &mut read_buf[..1]); // 16.500169500,203,0x9A,0xA3,Write,ACK
                                                                 // 16.500313500,203,0x9A,0x02,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.500675950,205,0x9A,0x80,Write,ACK
                                                           // 16.500995950,206,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0xa4, 0x00], &mut read_buf[..1]); // 16.501349200,208,0x9A,0xA4,Write,ACK
                                                                 // 16.501493200,208,0x9A,0x00,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.501857400,210,0x9A,0x80,Write,ACK
                                                           // 16.502177400,211,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x93, 0x11], &mut read_buf[..1]); // 16.502530950,213,0x9A,0x93,Write,ACK
                                                                 // 16.502674950,213,0x9A,0x11,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.503035600,215,0x9A,0x80,Write,ACK
                                                           // 16.503355600,216,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x93, 0x10], &mut read_buf[..1]); // 16.503712800,218,0x9A,0x93,Write,ACK
                                                                 // 16.503856800,218,0x9A,0x10,Write,ACK
    i2c.write_read(dac_addr, &[0x80], &mut read_buf[..1]); // 16.504426300,220,0x9A,0x80,Write,ACK
                                                           // 16.504746300,221,0x9B,0x00,Read,NAK
    i2c.write_read(dac_addr, &[0x82, 0x00], &mut read_buf[..1]); // 16.505107950,223,0x9A,0x82,Write,ACK
                                                                 // 16.505251950,223,0x9A,0x00,Write,ACK
    led.set_low();
    let mut lrclk_in = gpiob
        .pb9
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr);

    let mut bclk_in = gpiob
        .pb10
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut data_out = gpioc
        .pc3
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    let mut data = 0u16;
    loop {
        for inc in 200..400 {
            let mut dtmp = data;
            data += inc;
            while lrclk_in.is_low().unwrap() {}
            for i in 0..16 {
                while bclk_in.is_low().unwrap() {}

                if dtmp & (0b1000000000000000 >> i) != 0 {
                    data_out.set_high();
                } else {
                    data_out.set_low();
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
