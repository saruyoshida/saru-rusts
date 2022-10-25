#![no_std]
#![no_main]


use panic_halt as _;
use wio_terminal as wio;

use embedded_graphics::{
    prelude::*,
};

use cortex_m::interrupt::{free as disable_interrupts, CriticalSection};

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{interrupt, CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{button_interrupt, entry,  ButtonController, ButtonEvent, Pins, Sets};
use wio::hal::rtc;

use heapless::consts::U8;
use heapless::String;
use heapless::spsc::Queue;

use wio_clock::WioClock;
use wio_buttons::WioButtons;
use wio_toast::WioToast;

#[entry]
fn main() -> ! {
  let mut peripherals = 
    Peripherals::take().unwrap();
  let mut core = 
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

  let rtc = rtc::Rtc::clock_mode(
    peripherals.RTC, 
    1024.hz(), 
    &mut peripherals.MCLK
  );

  unsafe {
    RTC = Some(rtc);
  }

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

  let mut wio_clock = WioClock::new(
    &mut display
  );

  let button_ctrlr = sets.buttons.init(
      peripherals.EIC,
      &mut clocks,
      &mut peripherals.MCLK,
      &mut sets.port,
  );

  let nvic = &mut core.NVIC;

  disable_interrupts(|_| unsafe {
    button_ctrlr.enable(nvic);
    BUTTON_CTRLR = Some(button_ctrlr);
  });

  let mut consumer = unsafe { Q.split().1 };

  let mut wio_button = WioButtons::build(
    (0i32, 0i32, 23i32, 1i32, 
      String::from("hour")),
    (0i32, 0i32, 59i32, 1i32, 
      String::from("minute")),
    (0i32, 0i32, 59i32, 1i32, 
      String::from("second"))
  );

  let mut wio_toast = WioToast::new(
    3000u32,
    Point::new(260, 210),
    Size::new(60, 30),
    Rgb566::BLACK,
    Rgb565::WHITE,
    Rgb565::WHITE,
  );

  loop {
//        delay.delay_ms(1000 as u16);
    let time =
      disable_interrupts(|_| unsafe { 
        RTC.as_mut().map(|rtc| 
          rtc.current_time()
        ) 
      }
    ).unwrap();

    wio_clock.update(
     time.hours,
     time.minutes, 
     time.seconds
    );
    
    wio_clock.draw(&mut display).unwrap();

    wio_toast.count_down();
    wio_toast.draw(&mut display).unwrap();

    if let Some(press) = consumer.dequeue() {
      wio_button.reset_value(
        (time.hours as i32,
         time.minutes as i32,
         time.seconds as i32)
      );

      wio_toast.start(wip_buttons.get_state));
      
      if let Some(now_time) =
        wio_button.button_pulled(
        press.button) {
          set_time(now_time.0 as u8,
                   now_time.1 as u8,
                   now_time.2 as u8);
        }
      }
    }
}

static mut RTC: Option<rtc::Rtc<rtc::ClockMode>> = None;

fn set_time(hour: u8, minute: u8, second: u8) {
  disable_interrupts(|_| {
    unsafe {
      RTC.as_mut().map(|rtc| {
        rtc.set_time(rtc::Datetime {
          seconds: second,
          minutes: minute,
          hours: hour,
          day: 0,
          month: 0,
          year: 0,
        });
      });
    }
  });
}

static mut BUTTON_CTRLR: 
  Option<ButtonController> = None;
static mut Q: Queue<ButtonEvent, U8> =   
  Queue(heapless::i::Queue::new());

button_interrupt!(
  BUTTON_CTRLR,
  unsafe fn on_button_event(
    _cs: &CriticalSection, 
    event: ButtonEvent) {
      let mut q = Q.split().0;
      q.enqueue(event).ok();
    }
);
