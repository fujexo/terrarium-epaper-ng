#![no_std]
#![no_main]

use core::future::pending;
use defmt::{debug, error, info};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Delay;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use epd_waveshare::{epd4in2::*, prelude::*};
use u8g2_fonts::{
    fonts,
    types::{FontColor, VerticalPosition},
    FontRenderer,
};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_nrf::init(Default::default());
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M8;

    let cs_pin = Output::new(p.P1_03, Level::High, OutputDrive::Standard);
    let dc = Output::new(p.P1_04, Level::High, OutputDrive::Standard);
    let rst = Output::new(p.P1_05, Level::High, OutputDrive::Standard);
    let busy_in = Input::new(p.P1_06, embassy_nrf::gpio::Pull::None);

    let mut spi = spim::Spim::new_txonly(p.SPI3, Irqs, p.P1_02, p.P1_01, spi_config);

    let mut delay = Delay;

    debug!("Before EPD new");

    // Setup EPD
    let mut epd = match Epd4in2::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay) {
        Ok(epd) => epd,
        Err(err) => {
            error!("Failed to create EPD device: {}", err);
            panic!();
        }
    };
    debug!("After EPD New");

    // Use display graphics from embedded-graphics
    let mut display = Display4in2::default();
    display.set_rotation(DisplayRotation::Rotate180);

    // Use embedded graphics for drawing a line
    let _ = Line::new(Point::new(0, 0), Point::new(0, 300))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(399, 0), Point::new(399, 300))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(0, 0), Point::new(399, 0))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(0, 299), Point::new(399, 299))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display);

    if FontRenderer::new::<fonts::u8g2_font_inb24_mf>()
        .render(
            "Hello Rust!",
            Point::new(50, 100),
            VerticalPosition::Baseline,
            FontColor::Transparent(BinaryColor::On),
            &mut display,
        )
        .is_err()
    {
        error!("Failed to create text");
        panic!();
    };

    // Display updated frame
    epd.update_and_display_frame(&mut spi, display.buffer(), &mut delay)
        .expect("msg");

    // Set the EPD to sleep
    //epd.sleep(&mut spi, &mut delay).expect("epd sleep");

    info!("Updated Display, DONE :)");

    // Block forever so the above drivers don't get dropped
    pending::<()>().await;
}
