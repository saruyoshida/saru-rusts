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
use emb_shapegraph::*;
use emb_linegraph::*;

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
    (300_u32, 200_u32),  // 表示サイズ
    0..25,               // X目盛レンジ
    -200..1000,          // Y目盛レンジ
    (1.0, 1000.0),       // 補正率(x,y)
    (1, 100),            // 目盛刻み
    "title",             // タイトル
  );

  eb.set_base_color(BASE_COLOR)
    .set_text_color(TEXT_COLOR)
    .set_scale_color(SCALE_COLOR)
    .set_bar_color(BAR_COLOR)
    .set_box_color(BOX_COLOR);

  let mut es = EmbShapegraph::new(&eb);

  let mut ec = es.clone();
  ec.mode_circle()
    .set_shape_color(Rgb565::CYAN);

  let mut er = es.clone();
  er.mode_rectangle()
    .set_shape_color(BAR_COLOR);

  let mut et = es.clone();
  et.mode_triangle()
    .set_shape_color(SCALE_COLOR);

  let mut l1 = EmbLinegraph::new(&eb);
  l1.mode_dotline()
    .set_shape_color(SCALE_COLOR);

  let mut l2 = l1.clone();
  l2.mode_dotline()
    .set_shape_color(SCALE_COLOR);

  let mut l3 = l1.clone();
  l3.mode_realline()
    .set_shape_color(Rgb565::BLUE);
    
  let mut datas =  [0.104, 1.00, 0.050, 
                    0.299,0.397, 1.996, 
                    0.897, 0.799,0.601,
                    0.503,
                    0.104, 1.00, 0.050, 
                    0.299,0.397, 1.996, 
                    0.897, 0.799,0.601,
                    0.503,
                    0.299,0.397, 1.996, 
                    0.897, 0.799,];

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

  es.mode_revtriangle();

  loop {
    eb.mode_clear()
      .draw(&mut display)
      .unwrap();

    for (i, d) in datas.into_iter()
                       .enumerate()
    {
      es.set_data(i as f32, d)   
        .draw(&mut display)
        .unwrap();
      ec.set_data(i as f32, d + 0.05)
        .draw(&mut display)
        .unwrap();
      er.set_data(i as f32, d + 0.1)
        .draw(&mut display)
        .unwrap();
      et.set_data(i as f32, d - 0.05)
        .draw(&mut display)
        .unwrap();
      l3.set_data(i as f32, d)   
        .draw(&mut display)
        .unwrap();
      l1.set_data(i as f32, d + 0.3)
        .draw(&mut display)
        .unwrap();
      l2.set_data(i as f32, d - 0.3)
        .draw(&mut display)
        .unwrap();
    }
    l1.reset_data();
    l2.reset_data();
    l3.reset_data();

    datas.rotate_right(1);
    delay.delay_ms(5000 as u16);
  }
}

