#![no_std]
#![no_main]

use defmt::{debug, error, info};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::spi;
use embassy_stm32::time::Hertz;
use embassy_time::Delay;

use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use epd_waveshare::{epd4in2::*, prelude::*};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let cs_pin = Output::new(p.PE0, Level::High, Speed::VeryHigh);
    let dc = Output::new(p.PE2, Level::High, Speed::VeryHigh);
    let rst = Output::new(p.PE3, Level::High, Speed::VeryHigh);
    let busy_in = Input::new(p.PE1, Pull::None);

    //use core::cell::RefCell;
    //use embassy_sync::blocking_mutex::NoopMutex;
    //use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
    //let spi_bus = spi::Spi::new(p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH3, p.DMA1_CH2, spi_config);
    //let spi_bus = NoopMutex::new(RefCell::new(spi_bus));
    //let mut spi = SpiDevice::new(&spi_bus, cs_pin);

    let mut spi = spi::Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH3, p.DMA1_CH2, spi_config,
    );

    let mut delay = Delay;

    debug!("####################################################################################");

    // Setup EPD
    let mut epd = match Epd4in2::new(&mut spi, busy_in, dc, rst, &mut delay, None) {
        Ok(epd) => epd,
        Err(err) => {
            error!("Failed to create EPD device: {}", err);
            panic!();
        }
    };

    debug!("####################################################################################");

    // Use display graphics from embedded-graphics
    let mut display = Display4in2::default();

    // Use embedded graphics for drawing a line
    let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut display);

    // Display updated frame
    epd.update_frame(&mut spi, display.buffer(), &mut delay)
        .expect("update frame");
    epd.display_frame(&mut spi, &mut delay)
        .expect("display frame");

    // Set the EPD to sleep
    //epd.sleep(&mut spi, &mut delay).expect("epd sleep");

    info!("Updated Display, DONE :)");
}
