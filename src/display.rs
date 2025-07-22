use embedded_graphics_core::{pixelcolor::BinaryColor, prelude::*};
use esp_hal::{
    delay::Delay,
    gpio::Output,
    spi::{
        master::{Address, Command, Spi},
        DataMode,
    },
    Blocking,
};
use bitset_core::BitSet;

const OFFSET: usize = 4;

const ROWS: usize = 7;
const COLS: usize = 12 * 15;

// Each column is packed into 8 bit ints with the offset
const COL_LEN_BYTES: usize = (COLS + OFFSET) / 8 + 1;

pub struct LcdDisplay {
    buf: [[u8; COL_LEN_BYTES]; ROWS],
    delay: Delay,
    rows: [Output<'static>; ROWS],
    spi: Spi<'static, Blocking>,
}

impl LcdDisplay {
    pub fn new(
        rows: [Output<'static>; ROWS],
        spi: Spi<'static, Blocking>,
    ) -> Self {
        Self {
            buf: [[0; COL_LEN_BYTES]; ROWS],
            delay: Delay::new(),
            rows,
            spi,
        }
    }

    pub fn clear(&mut self) {
        self.buf = [[0; COL_LEN_BYTES]; ROWS];
    }

    pub fn flush(&mut self) {
        for row in 0..ROWS {
            let row_pin = self.rows.get_mut(row).unwrap();
            row_pin.set_high();
            self.spi
                .half_duplex_write(
                    DataMode::Single,
                    Command::None,
                    Address::None,
                    0,
                    &self.buf[row],
                )
                .unwrap();
            self.delay.delay_micros(1000);
            row_pin.set_low();
        }
    }
}

#[derive(Debug)]
pub enum LcdDisplayError {}

impl DrawTarget for LcdDisplay {
    type Color = BinaryColor;
    type Error = LcdDisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, colour) in pixels {
            if (0..(COLS as i32)).contains(&coord.x) && (0..(ROWS as i32)).contains(&coord.y) {
                if colour.is_on() {
                    self.buf[coord.y as usize].bit_set(coord.x as usize + OFFSET);
                } else {
                    self.buf[coord.y as usize].bit_reset(coord.x as usize + OFFSET);
                }
            }
        }

        Ok(())
    }
}

impl OriginDimensions for LcdDisplay {
    fn size(&self) -> Size {
        Size::new(COLS as u32, ROWS as u32)
    }
}
