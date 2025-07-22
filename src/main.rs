#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod display;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use embedded_vintage_fonts::FONT_6X8;

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::Spi;
use esp_hal::spi::{self, BitOrder};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use log::info;

use esp_backtrace as _;

use crate::display::LcdDisplay;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK)
        .expect("Failed to initialize WIFI/BLE controller");
    let (mut _wifi_controller, _interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");

    // TODO: Spawn some tasks
    let _ = spawner;

    let rows_config = OutputConfig::default();
    let level = Level::Low;
    let rows = [
        Output::new(peripherals.GPIO14, level, rows_config),
        Output::new(peripherals.GPIO13, level, rows_config),
        Output::new(peripherals.GPIO12, level, rows_config),
        Output::new(peripherals.GPIO11, level, rows_config),
        Output::new(peripherals.GPIO10, level, rows_config),
        Output::new(peripherals.GPIO9, level, rows_config),
        Output::new(peripherals.GPIO8, level, rows_config),
    ];

    // Active low
    let _col_clr = Output::new(peripherals.GPIO38, Level::High, OutputConfig::default());
    // Active low
    let _enable = Output::new(peripherals.GPIO17, Level::Low, OutputConfig::default());

    let spi_config = spi::master::Config::default()
        .with_frequency(Rate::from_khz(5000))
        .with_write_bit_order(BitOrder::LsbFirst)
        // Idle high. Propagate on rising edge
        .with_mode(spi::Mode::_0);

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO21);

    let mut lcd = LcdDisplay::new(rows, spi);

    let text_style = MonoTextStyle::new(&FONT_6X8, BinaryColor::On);

    loop {
        let text = Text::new(
            "BORNHACK 2025 HUGE DISPLAY :-)",
            Point::new(0, 6),
            text_style,
        );

        lcd.clear();
        text.draw(&mut lcd).unwrap();
        lcd.flush();
    }
}
