#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use micromath::F32Ext;
use core::fmt::Write;

use emb_bargraph::*;
use emb_shapegraph::*;
use emb_linegraph::*;
use one_dimensional::*;
use emb_textterm::*;
use dogsimulation::DogSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const DOT_COLOR  : Rgb565 = Rgb565::WHITE;
const TRIA_COLOR : Rgb565 = Rgb565::RED;
const CIRC_COLOR : Rgb565 = Rgb565::WHITE;
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

  // グラフ設定
  let mut eb = bargraph_setting();
  // 文字表示設定
  let mut et = textterm_setting();
  // 図形グラフ表示設定
  let mut es = EmbShapegraph::new(&eb);
  // フィルタライン表示設定
  let mut lm = EmbLinegraph::new(&eb);
               lm.set_shape_color(LINE_COLOR);
  // 分散上ライン表示設定
  let mut lu = lm.clone();
               lu.mode_dotline()
                 .set_shape_color(DOT_COLOR);
  // 分散下ライン表示設定
  let mut ld = lu.clone();
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
  eb.mode_clear().draw(&mut display).unwrap();
  eb.mode_scale().draw(&mut display).unwrap();
  // テキスト表示領域クリア
  et.mode_clear().draw(&mut display).unwrap();
  et.mode_data();
// ----------------------------------------
// ----------------------------------------
  // フィルタ関数
//  // 1.ガウス分布の積:ベイズ的アプローチ
//  let filter_fn = OneDimKalman; 
  // 2.残差カルマンゲイン形式
  let filter_fn = OneDimKalmanRest;
  // 設定値
  let process_var = 2.0; // 犬の動きの分散
  let sensor_var  = 4.5; // センサーの分散
  let x  = (0.0, 400.0); // 犬の初期推定位置
  let velocity    = 1.0; // 単位移動量
  let dt          = 1.0; // タイムステップ        
  // シミュレータ
  let mut target = DogSimulation::new();
  target.set_random_seed(134)
        .set_process_var(process_var)                     
        .set_measurement_var(sensor_var)
        .set_x(x.0)
        .set_velocity(velocity);
  // フィルタ
  let mut obfilter = ObjObsFilter::new(
    target,
    filter_fn
  );
  obfilter.set_process_model(
             (velocity * dt, process_var)
           )
          .set_sensor_var(sensor_var)
          .set_x(x)
          .set_dt(dt);

  // 繰返し観測
  for i in 0..25 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    let z = obfilter.zs(); // ノイズ有観測値
                           // 事前予測値,分散
    let (p_mean, p_var) = obfilter.prior();
                           // 事後予測値
    let (x_mean, x_var) = obfilter.xs();
    // 事前予測値表示
    es.mode_revtriangle()
      .set_shape_color(TRIA_COLOR)
      .set_data(i as f32, p_mean)
      .draw(&mut display)
      .unwrap();
    // 観測値表示
    es.mode_circle()
      .set_shape_color(CIRC_COLOR)
      .set_data(i as f32, z)
      .draw(&mut display)
      .unwrap();
    // フィルタ結果表示
    lm.set_data(i as f32, x_mean)
      .draw(&mut display)
      .unwrap();
    // 事前予測分散表示
    let stdv = p_var.sqrt();
      // 上
    lu.set_data(i as f32, x_mean + stdv)
      .draw(&mut display)
      .unwrap();
      // 下
    ld.set_data(i as f32, x_mean - stdv)
      .draw(&mut display)
      .unwrap();
    // 文字列
    let dsp_text = ontheway_text(
      p_mean, p_var, z, x_mean,x_var,
    );
    et.set_data(dsp_text)   
      .draw(&mut display)
      .unwrap();
    // ウエイト
    delay.delay_ms(500 as u16);
  }
  // 終了
  loop {}
}
// ----------------------------------------
// ----------------------------------------
// 途中結果文字列表示
fn ontheway_text(
  p_mean : f32,
  p_var  : f32,
  z      : f32,
  x_mean : f32,
  x_var  : f32,
) -> EttString
{
  let mut dsp_text = EttString::new();
  dsp_text.clear();
  writeln!(
    dsp_text, 
    "{:>03.03} {:>03.03} {:>03.03} {:>03.03} {:>03.03}", 
    p_mean,
    p_var,
    z,
    x_mean,
    x_var,
  ).unwrap();

  dsp_text
}
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 160_u32),  // 表示サイズ
    0..25,               // X目盛レンジ
    -2000..4000,         // Y目盛レンジ
    (1.0, 100.0),        // 補正率(x,y)
    (5, 1000),           // 目盛刻み
                         // タイトル
    ":Filter,:Measurements,:prediction",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
// 文字表示セッティング
fn textterm_setting() -> EmbTextterm<'static>
{
  let mut textterm = EmbTextterm::new(
    (0, 161),             // 表示開始位置
    (320_u32, 79_u32),   // 表示サイズ
  );
  textterm.set_base_color(BASE_COLOR)
          .set_text_color(TEXT_COLOR)
          .set_txt2_color(TXT2_COLOR)
          .set_box_color(BOX_COLOR);
  textterm
}