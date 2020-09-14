#![no_std]

// #[macro_use]
extern crate stm32l4xx_hal as hal;
pub mod pcm5122;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;

pub fn flare_led<PIN: OutputPin<Error = E>, E>(led: &mut PIN, timer: &mut Delay) -> Result<(), E> {
    led.set_high()?;
    timer.delay_ms(1u32);
    led.set_low()?;
    timer.delay_ms(1u32);
    led.set_high()?;
    timer.delay_ms(1u32);
    led.set_low()?;
    timer.delay_ms(1u32);
    Ok(())
}
