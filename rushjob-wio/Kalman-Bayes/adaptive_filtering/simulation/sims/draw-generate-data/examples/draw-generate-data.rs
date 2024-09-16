#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use generate_data::GenerateData;

use emb_bargraph::*;
use emb_shapegraph::*;
use emb_linegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const ELLP_COLOR : Rgb565 = Rgb565::YELLOW;

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
// ----------------------------------------
// グラフ表示設定
  // グラフ設定
  let mut eb = bargraph_setting();
  // 観測値表示設定
  let mut sz = EmbShapegraph::new(&eb);
               sz.mode_circle()
                 .set_shape_color(ELLP_COLOR);
  // 表示設定
  let mut lp = EmbLinegraph::new(&eb);
               lp.set_shape_color(LINE_COLOR)
                 .set_shape_width(2);
  // 画面クリア
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();
  // 目盛表示
  eb.mode_scale().draw(&mut display).unwrap();
// ========================================
// シミュレーションデータ表示
  let gd = GenerateData::new(50, 2.);
  for (pos, z) in gd.into_iter() {
// ----------------------------------------
// グラフ表示
    // 実際位置表示
    lp.set_data(pos.0, pos.1)
      .draw(&mut display)
      .unwrap();
// ========================================
    // 観測値表示
    sz.set_data(z.0, z.1)
      .draw(&mut display)
      .unwrap();
  }
  // 終了
  loop {}
}
// ----------------------------------------
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (5, 5),              // 表示開始位置
    (310_u32, 230_u32),  // 表示サイズ
    -400..600,           // X目盛レンジ
    -100..500,           // Y目盛レンジ
    (10.0, 10.0),        // 補正率(x,y)
    (200, 100),          // 目盛刻み
                         // タイトル
    "",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
