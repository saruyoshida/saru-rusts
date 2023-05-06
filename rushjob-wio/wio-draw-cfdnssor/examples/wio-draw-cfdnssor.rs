#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::
  {PrimitiveStyle, Rectangle};

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use wio_cfdnssor::CfdNsSor;

const BG_COLOR: Rgb565 = Rgb565::WHITE;

#[entry]
fn main() -> ! {
  let mut peripherals = 
    Peripherals::take().unwrap();
  let core = 
    CorePeripherals::take().unwrap();

  let mut clocks = GenericClockController::
    with_external_32kosc(
      peripherals.GCLK,
      &mut peripherals.MCLK,
      &mut peripherals.OSC32KCTRL,
      &mut peripherals.OSCCTRL,
      &mut peripherals.NVMCTRL,
  );

  let mut delay = Delay::new(
    core.SYST, &mut clocks);
  let pins = Pins::new(peripherals.PORT);
  let mut sets: Sets = pins.split();

  let (mut display, _backlight) = 
    sets.display
      .init(
        &mut clocks,
        peripherals.SERCOM7,
        &mut peripherals.MCLK,
        &mut sets.port,
        58.mhz(),
        &mut delay,
      )
      .unwrap();

  let mut wio_cfdnssor = CfdNsSor::new();

  Rectangle::new(
    Point::new(0, 0), 
    Size::new(320, 240),
  )
  .into_styled(
    PrimitiveStyle::with_fill(BG_COLOR)
  )
  .draw(&mut display).unwrap();

  loop {
    wio_cfdnssor.update();

    wio_cfdnssor
      .draw_off()
      .draw(&mut display).unwrap();

    wio_cfdnssor
      .draw_on()
      .draw(&mut display).unwrap();
  }
}



