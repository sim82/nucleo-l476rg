#![deny(unsafe_code)]
// #![deny(warnings)]

extern crate cortex_m;

// #[macro_use(block)]
// extern crate nb;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c::{Write, WriteRead};

pub struct Pcm5122<I2C> {
    i2c: I2C,
    dac_addr: u8,
}

impl<I2C, E> Pcm5122<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Pcm5122 {
            i2c,
            dac_addr: 0x9au8 >> 1,
        }
    }

    pub fn write_register(&mut self, reg: usize, value: usize) -> Result<(), E> {
        let mut read_buf = [0u8; 1];
        self.i2c
            .write_read(self.dac_addr, &[0x80], &mut read_buf[..1])?;
        let reg = (reg | 0x80) as u8;
        self.i2c
            .write_read(self.dac_addr, &[reg as u8, value as u8], &mut read_buf)?;
        Ok(())
    }

    pub fn read_register(&mut self, reg: usize) -> Result<usize, E> {
        let mut read_buf = [0u8; 1];
        self.i2c
            .write_read(self.dac_addr, &[0x80], &mut read_buf[..1])?;
        let reg = (reg | 0x80) as u8;
        self.i2c
            .write_read(self.dac_addr, &[reg as u8], &mut read_buf)?;
        Ok(read_buf[0] as usize)
    }
}

pub fn init_default<I2C, E>(
    pcm5122: &mut Pcm5122<I2C>,
    timer: &mut dyn DelayMs<u16>,
) -> Result<(), E>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    // 1.679802350,3,0x9A,0x81,Write,ACK reset RSTM + RSTR
    // 1.679946350,3,0x9A,0x11,Write,ACK
    pcm5122.write_register(0x1, 0x11)?;
    // 1.683189700,13,0x9A,0x10,Write,ACK standby mode on, powerdown off
    // 1.683590700,15,0x9A,0x80,Write,ACK
    pcm5122.write_register(0x1, 0x0)?;
    //1.683189700,13,0x9A,0x10,Write,ACK standby mode on, powerdown off
    pcm5122.write_register(0x2, 0x10)?;
    // 1.684302450,18,0x9A,0x82,Write,ACK
    // 1.684446450,18,0x9A,0x11,Write,ACK standby mode on, powerdown on
    pcm5122.write_register(0x2, 0x11)?;
    // 1.687804250,29,0x9A,0x88,Write,ACK
    // 1.687948250,29,0x9A,0x24,Write,ACK set GPIO 3 & 6 output => xtal enable
    pcm5122.write_register(0x8, 0x24)?;
    // 1.690474750,40,0x9A,0xD2,Write,ACK gpio 3 register output
    // 1.690618750,40,0x9A,0x02,Write,ACK
    pcm5122.write_register(0x52, 0x02)?;
    // 1.693157450,51,0x9A,0xD5,Write,ACK gpio 6 register output
    // 1.693301450,51,0x9A,0x02,Write,ACK
    pcm5122.write_register(0x55, 0x02)?;
    // 1.695695300,62,0x9A,0xD6,Write,ACK
    // 1.695839300,62,0x9A,0x20,Write,ACK set gpio6 high (22.5792 MHz xtal -> 44.1k)
    pcm5122.write_register(0x56, 0x20)?;
    timer.delay_ms(100u16);
    // 1.725111400,73,0x9A,0xD6,Write,ACK GPIO ouput control, all low
    // 1.725255400,73,0x9A,0x00,Write,ACK
    pcm5122.write_register(0x56, 0x00)?;
    // 1.755291600,84,0x9A,0xD6,Write,ACK set gpio3 high (24.576 MHz xtal -> 48k)
    // 1.755435600,84,0x9A,0x04,Write,ACK
    pcm5122.write_register(0x56, 0x04)?;
    timer.delay_ms(30u16);
    // 1.785154850,95,0x9A,0x89,Write,ACK BCK & LRCK master (yay!)
    // 1.785298850,95,0x9A,0x11,Write,ACK
    pcm5122.write_register(0x9, 0x11)?;
    // 1.786352450,100,0x9A,0x8C,Write,ACK BCK & LRCK divider functional
    // 1.786496450,100,0x9A,0x7F,Write,ACK
    pcm5122.write_register(0xc, 0x7f)?;
    // BCK & LRCK start
    // 1.787518300,105,0x9A,0xA1,Write,ACK master mode LRCK divider
    // 1.787662250,105,0x9A,0x3F,Write,ACK divide by 64
    // LRCK 4MHz -> 384KHz
    pcm5122.write_register(0x21, 0x3f)?;
    // 1.788684300,110,0x9A,0x88,Write,ACK set GPIO 3, 4 & 6 output => xtal enable + LED
    // 1.788828300,110,0x9A,0x2C,Write,ACK
    pcm5122.write_register(0x8, 0x2c)?;
    // 1.791311400,121,0x9A,0xD3,Write,ACK gpio 4 register output
    // 1.791455400,121,0x9A,0x02,Write,ACK
    pcm5122.write_register(0x53, 0x02)?;
    // 1.792480100,126,0x9A,0xD6,Write,ACK gpio 3 & 4 high (led ON)
    // 1.792624100,126,0x9A,0x0C,Write,ACK
    pcm5122.write_register(0x56, 0x0c)?;
    // 16.467988200,131,0x9A,0x82,Write,ACK request standby mode
    // 16.468132200,131,0x9A,0x10,Write,ACK
    pcm5122.write_register(0x2, 0x10)?;
    // 16.474832550,136,0x9A,0xD6,Write,ACK gpio 6 high, gpio 3 low -> (back to 22.5792 MHz xtal)
    // 16.474976550,136,0x9A,0x28,Write,ACK
    pcm5122.write_register(0x56, 0x28)?;
    // 16.477386700,147,0x9A,0xA8,Write,ACK I2S data format, 16bit
    // 16.477530700,147,0x9A,0x00,Write,ACK
    pcm5122.write_register(0x28, 0x00)?;
    // 16.488012650,152,0x9A,0xA5,Write,ACK IPLK 1, DCAS 1, IDCM 0, IDCH 1, IDSK 1, IDBK 1, IDFS 1 (ignore clock errors ?)
    // 16.488156650,152,0x9A,0x7B,Write,ACK
    pcm5122.write_register(0x25, 0x7b)?;
    // 16.489209450,157,0x9A,0x84,Write,ACK PLL disable
    // 16.489529450,158,0x9B,0x01,Read,NAK
    pcm5122.write_register(0x4, 0x00)?;
    // 16.491763600,168,0x9A,0x8E,Write,ACK DAC clock source is SCK
    // 16.491907600,168,0x9A,0x30,Write,ACK
    pcm5122.write_register(0xe, 0x30)?;
    // 16.492994350,173,0x9A,0x9B,Write,ACK DSP clock divider = 1
    // 16.493138350,173,0x9A,0x00,Write,ACK
    pcm5122.write_register(0x1b, 0x00)?;
    // 16.494202750,178,0x9A,0x9C,Write,ACK
    // 16.494346750,178,0x9A,0x03,Write,ACK DAC clock divider = 4
    pcm5122.write_register(0x1c, 0x03)?;
    // 16.495420750,183,0x9A,0x9D,Write,ACK
    // 16.495564750,183,0x9A,0x03,Write,ACK NCP clock divider = 4
    pcm5122.write_register(0x1d, 0x03)?;
    // 16.496618550,188,0x9A,0x9E,Write,ACK
    // 16.496762550,188,0x9A,0x07,Write,ACK OSR clock divider = 8
    pcm5122.write_register(0x1e, 0x07)?;
    // 16.497811650,193,0x9A,0xA0,Write,ACK
    // 16.497955650,193,0x9A,0x0F,Write,ACK master mode BCK divider = 16
    pcm5122.write_register(0x20, 0x0f)?;
    // 16.498994800,198,0x9A,0xA1,Write,ACK
    // 16.499138800,198,0x9A,0x1F,Write,ACK master mode LRCK divider = 32
    pcm5122.write_register(0x21, 0x1f)?;
    // 16.500169500,203,0x9A,0xA3,Write,ACK ? undocumented?
    // 16.500313500,203,0x9A,0x02,Write,ACK
    pcm5122.write_register(0x23, 0x02)?;
    // 16.501349200,208,0x9A,0xA4,Write,ACK
    // 16.501493200,208,0x9A,0x00,Write,ACK DSP clock cycles per frame (0?)
    pcm5122.write_register(0x24, 0x00)?;
    // 16.502530950,213,0x9A,0x93,Write,ACK
    // 16.502674950,213,0x9A,0x11,Write,ACK halt DAC
    pcm5122.write_register(0x13, 0x11)?;
    // 16.503712800,218,0x9A,0x93,Write,ACK
    // 16.503856800,218,0x9A,0x10,Write,ACK resume DAC
    pcm5122.write_register(0x13, 0x10)?;
    // 16.505107950,223,0x9A,0x82,Write,ACK
    // 16.505251950,223,0x9A,0x00,Write,ACK resume from stansby
    pcm5122.write_register(0x2, 0x00)?;
    Ok(())
}
