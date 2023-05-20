#![no_std]
#![no_main]


use panic_halt as _;
use wio_terminal as wio;

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::
  {PrimitiveStyle, Rectangle};

use cortex_m::interrupt::{free as disable_interrupts, CriticalSection};

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{interrupt, CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{button_interrupt, ButtonEvent, entry,  ButtonController, Pins, Sets};
use heapless::consts::*;
// 削除
// use heapless::String;
//
use heapless::spsc::Queue;
use heapless::Vec;

use wio_elldiski::WioElliptClock;
// 変更
// use wio_buttons::{WioButtons};
use wio_sbbutton::WioSBButton;
//

// 削除
// use wio_toast::WioToast;
//

use wio_polywave::WioPolyWave;

// 追加
use wio_sbcamera::WioSBCamera;
//

// 削除
// const LINE_COLOR: Rgb565 = Rgb565::WHITE;
// const CENTER_COLOR: Rgb565 = Rgb565::BLUE;
//
const BG_COLOR: Rgb565 = Rgb565::BLACK;
const APEX: i32 = 10;

const DISKS: i32 = 10;
const RADIUS: i32 = 60;
const SPEED: usize = 6;

// 追加
pub trait SBCConvertTrait {
  fn draw_points(&mut self) 
    -> &mut [Point];

  fn convert(
    &mut self, 
    camera: &mut WioSBCamera
  ) -> &mut Self
  {
    camera.convert(self.draw_points());
    self
  }
}

impl SBCConvertTrait for WioElliptClock {
  fn draw_points(&mut self) -> &mut [Point]
  {
    self.as_mut_points()
  }
}

impl SBCConvertTrait for WioPolyWave {
  fn draw_points(&mut self) -> &mut [Point]
  {
    self.as_mut_points()
  }
}
//

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

// 変更
//  let mut wio_button = WioButtons::build(
//    (APEX, 1i32, APEX * 2 - 1 , 1i32, 
//      String::from("axis")),
//    (60i32, 1i32, 60i32, 1i32, 
//      String::from("dp")),
//    (0i32, 0i32, 1i32, 1i32, 
//      String::from("ce"))
//  );
  let mut wio_button = WioSBButton::new(320);
//

// 削除
//  let now_value = wio_button.get_value();

// 追加
  // 通常
  let now_value = (APEX , RADIUS, 0);
  // 振動かつ三角形
//  let now_value = (APEX / 5, 3, 1);

  let  (a, b) = get_apex(now_value.0);

  let mut camera = 
    WioSBCamera::build_default(320.0, 240.0);

  camera.set_eye(wio_button.position());
//

  let mut disks = Vec::<WioElliptClock, U10>
                  ::new();

  for i in 0..DISKS {
    let (ctk, r) = get_ctkr(i + 1);
    let wio_clock = WioElliptClock::new(
      ctk,           // 進度係数
      r as f32,          // 半径
    );
    let _ = disks.push(wio_clock);
  }

  disks.iter_mut().for_each(|wio_clock|
    wio_clock.reset(
// 変更
//      1.0f32, 
//      1.0f32,
      a,
      b,
//
      now_value.1,
      now_value.2
    )
  );

// 削除
//  let mut wio_toast = WioToast::new(
//    180,
//    Point::new(260, 210),
//    Size::new(60, 20),
//    LINE_COLOR,
//    BG_COLOR,
//    CENTER_COLOR,
//  );

  let mut wio_polywave = WioPolyWave::new(
    160,320
  );

  Rectangle::new(
    Point::new(0, 0), 
    Size::new(320, 240),
  )
  .into_styled(
    PrimitiveStyle::with_fill(
      BG_COLOR)
  )
  .draw(&mut display).unwrap();


  let mut ct = (0..360).cycle()
                       .step_by(SPEED);

  loop {
    let ctr = ct.next().unwrap();

    let mut x0y0 = (80, 120);
    let mut th = 0;

    for wio_clock in disks.iter_mut() {
      let (x1y1, th1)
        = wio_clock.update(ctr, x0y0, th);
       x0y0 = x1y1;
       if now_value.2 < 2 {
         th = 0;
       } else {
         th = th1;
       }
    }

    wio_polywave.update(x0y0);
// 追加
    wio_polywave.swap_start();
//
    wio_polywave.draw_off()
             .draw(&mut display).unwrap();

    disks.iter_mut().for_each(|wio_clock|
      wio_clock.draw_off()
               .draw(&mut display).unwrap()
    );

    disks.iter_mut().for_each(|wio_clock|
      wio_clock.draw_on()
// 追加
               .convert(&mut camera)
//
               .draw(&mut display).unwrap()
    );
    
    wio_polywave.draw_on()
// 追加
             .convert(&mut camera)
//
             .draw(&mut display).unwrap();

// 追加
    wio_polywave.swap_end();
//

// 削除
//    wio_toast.count_down()
//             .draw(&mut display).unwrap();

    if let Some(press) = consumer.dequeue() {
// 削除
//      wio_toast.start(
//        String::from(
//          wio_button.get_state().as_str()
//        )
//      );
//
//      if let Some(now_value) =
//

// 追加
      camera.set_eye(
//
        wio_button.button_pulled(
        press.button) 
// 追加
      );
//

// 削除
//    {
//      let  (a, b) = get_apex(now_value.0)
//
//      disks.iter_mut().for_each(|wio_clock|
//        wio_clock.reset(
//          a, 
//          b,
//          now_value.1,
//          now_value.2
//        )
//      );
//    }
//
    }
  }
}

fn get_apex(apex: i32) -> (f32, f32) {
  let a = if apex >= APEX {
            1f32
          } else {
            apex as f32 / APEX as f32
          };
  let b = if apex <= APEX {
            1f32
          } else {
            (APEX * 2 - apex) as f32 
            / APEX as f32
          };
  (a, b)
}

fn get_ctkr(i: i32) -> (i32, i32) {
  let ctk = i * 2 - 1;
  let r = RADIUS / ctk;
  (ctk, r)
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
