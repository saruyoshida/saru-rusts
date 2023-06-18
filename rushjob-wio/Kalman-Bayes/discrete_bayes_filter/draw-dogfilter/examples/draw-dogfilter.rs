#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use core::fmt::Write;

use emb_bargraph::*;
use emb_textterm::*;
// フィルタ指定
use dog_filter::ObjObsFilter;

// 表示設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const BAR_COLOR  : Rgb565 = Rgb565::CYAN;
const BAR2_COLOR : Rgb565 = Rgb565::YELLOW;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const TEXT_COLOR : Rgb565 = Rgb565::WHITE;
const TXT2_COLOR : Rgb565 = Rgb565::CYAN;

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

  // 棒グラフ設定(prior)
  let mut e2 = EmbBargraph::new(
    (1, 121),            // 表示開始位置
    (158_u32, 118_u32),  // 表示サイズ
    0..10,               // X目盛レンジ
    0..500,              // Y目盛レンジ
    (1.0, 1000.0),       // 補正率(x,y)
    (1, 100),            // 目盛刻み
    "prior",             // タイトル
  );
  prior_colorsetting(&mut e2);
  // 棒グラフ設定(posterior)
  let mut eb = EmbBargraph::new(
    (159, 121),          // 表示開始位置
    (158_u32, 118_u32),  // 表示サイズ
    0..10,               // X目盛レンジ
    0..500,             // Y目盛レンジ
    (1.0, 1000.0),       // 補正率(x,y)
    (1, 100),            // 目盛刻み
    "posterior",         // タイトル
  );
  posterior_colorsetting(&mut eb);
  // テキスト表示設定
  let mut et = EmbTextterm::new(
    (10, 1),             // 表示開始位置
    (300_u32, 118_u32),  // 表示サイズ
  );
  text_colorsetting(&mut et);
  // 画面クリア
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();
  // 棒グラフ領域クリア、目盛表示
  e2.mode_clear().draw(&mut display).unwrap();
  e2.mode_scale().draw(&mut display).unwrap();
  eb.mode_clear().draw(&mut display).unwrap();
  eb.mode_scale().draw(&mut display).unwrap();
  // テキスト表示領域クリア
  et.mode_clear().draw(&mut display).unwrap();
  et.mode_data();
// ----------------------------------------
// ----------------------------------------
  // 観測対象フィルタビルド
  let mut obfilter = ObjObsFilter::new();
  // 繰返し数設定
  let loop_cnt     = obfilter.loop_cnt();
  // 表示開始回数
  let dsp_cnt      = obfilter.dsp_cnt();
  // 繰返し観測
  for i in 0..loop_cnt {
    // 観測対象フィルタ実行
    obfilter.iterations();
    // 途中結果表示
    if i >= dsp_cnt {
      // 文字列
      let dsp_text = ontheway_text(
        &obfilter,
        i,
      );
      et.set_data(dsp_text)   
        .draw(&mut display)
        .unwrap();
      // 棒グラフ
      //   領域クリア
      e2.mode_clear();
      e2.draw(&mut display).unwrap();
      eb.mode_clear();
      eb.draw(&mut display).unwrap();
      //   表示(prior)
      e2.mode_data();
      for (i, d) in obfilter.prior()
                            .iter()
                            .enumerate() {
        e2.set_data(i as f32, *d)
          .draw(&mut display)
          .unwrap();
      }
      //   表示(posterior)
      eb.mode_data();
      for (i, d) in obfilter.posterior()
                            .iter()
                            .enumerate() {
        eb.set_data(i as f32, *d)   
          .draw(&mut display)
          .unwrap();
      }
    }
    // ウエイト
    delay.delay_ms(1000 as u16);
  }
  // 終了
  loop {}
}
// ----------------------------------------
// ----------------------------------------

// 途中結果文字列表示
fn ontheway_text(
  obfilter : &ObjObsFilter,
  i            : i32,
) -> EttString
{
  let mut dsp_text = EttString::new();
//  let (index, val) = obfilter.argmax();
  dsp_text.clear();
  writeln!(
    dsp_text, 
    "Time {}: DorOrWall {}", 
    i,
//    obfilter.pos(),
    obfilter.sensor_pos(),
//    index,
//    val * 100.0,
  ).unwrap();

  dsp_text
}

fn prior_colorsetting(
  bargraph: &mut EmbBargraph
)
{
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(TEXT_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_bar_color(BAR2_COLOR)
          .set_box_color(BOX_COLOR);
}

fn posterior_colorsetting(
  bargraph: &mut EmbBargraph
)
{
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(TEXT_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_bar_color(BAR_COLOR)
          .set_box_color(BOX_COLOR);
}
fn text_colorsetting(
  textterm: &mut EmbTextterm
)
{
  textterm.set_base_color(BASE_COLOR)
         .set_text_color(TEXT_COLOR)
         .set_txt2_color(TXT2_COLOR)
         .set_box_color(BOX_COLOR);
}