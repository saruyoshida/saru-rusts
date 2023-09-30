#![no_std]
#![no_main]

// バー表示テスト

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use emb_bargraph::*;
use emb_covargraph::*;

// 表示設定
const BASE_COLOR : Rgb565 = Rgb565::WHITE;
const BAR_COLOR  : Rgb565 = Rgb565::YELLOW;
const SCALE_COLOR: Rgb565 = Rgb565::BLACK;
const BOX_COLOR  : Rgb565 = Rgb565::WHITE;
const TEXT_COLOR : Rgb565 = Rgb565::BLACK;

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

  let mut eb = EmbBargraph::new(
    (10, 10),            // 表示開始位置
    (180_u32, 240_u32),  // 表示サイズ
    -30..70,             // X目盛レンジ
    0..150,              // Y目盛レンジ
    (10.0, 10.0),        // 補正率(x,y)
    (10, 20),            // 目盛刻み
    "title",             // タイトル
  );

  eb.set_base_color(BASE_COLOR)
    .set_text_color(TEXT_COLOR)
    .set_scale_color(SCALE_COLOR)
    .set_bar_color(BAR_COLOR)
    .set_box_color(BOX_COLOR);

  let mut eg = EmbCovargraph::new(&eb);
  eg.mode_realline()
    .set_shape_color(Rgb565::BLUE)
    .set_shape_width(2)
    .set_std(&[1., 2., 3.]);
  
    
  let datas     =  [
    (Vector2::new(2.0,  7.0),
     Matrix2::new(2.0, 0.0, 0.0,2.0)),
    (Vector2::new(2.0,  7.0),
     Matrix2::new(2.0, 0.0, 0.0, 6.0)),
    (Vector2::new(2.0,  7.0),
     Matrix2::new(2.0, 1.2, 1.2, 2.0)),
];

  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();

  eb.mode_allclear()
    .draw(&mut display)
    .unwrap();
  
  eb.mode_scale().draw(&mut display).unwrap();

  loop {
    for (i, d) in datas.into_iter()
    {
      eb.mode_clear()
        .draw(&mut display)
        .unwrap();

      eg.set_data(i.as_slice(), d.as_slice())
        .draw(&mut display)
        .unwrap();
      delay.delay_ms(1000 as u16);
    }
//    datas.rotate_right(1);
  }
}

