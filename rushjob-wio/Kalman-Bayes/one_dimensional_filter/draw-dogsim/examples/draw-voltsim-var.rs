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

use emb_bargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;
use one_dimensional::*;
use emb_gaussgraph::*;
use dogsimulation::DogSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const P_COLOR    : Rgb565 = Rgb565::GREEN;
const Z_COLOR    : Rgb565 = Rgb565::RED;
const X_COLOR    : Rgb565 = Rgb565::BLUE;

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
  // ガウスグラフ設定
  let mut e2 = bar2graph_setting();
  let mut eg = EmbGaussgraph::new(&e2);
  eg.mode_realline()
    .set_shape_color(X_COLOR);
  // 図形グラフ表示設定
  let mut es = EmbShapegraph::new(&eb);
  // フィルタライン表示設定
  let mut lf = EmbLinegraph::new(&eb);
               lf.set_shape_color(X_COLOR);
  // 予測値ライン
  let mut lp = lf.clone();
               lp.mode_dotline()
                 .set_shape_color(P_COLOR);
  // 残差ライン表示設定
  let mut lz = lf.clone();
               lz.mode_dotline()
                 .set_shape_color(Z_COLOR);
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
  // ガウスグラフ表示領域クリア、目盛表示
  e2.mode_clear().draw(&mut display).unwrap();
  e2.mode_scale().draw(&mut display).unwrap();
// ----------------------------------------
// ----------------------------------------
  // 2.残差カルマンゲイン形式
  let filter_fn = OneDimKalmanRest;
  // 設定値
  let process_var    = 0.05;
  let voltage_std    = 2.5;
  let actual_voltage = 16.3;
  let x              = (25.0, 1000.0);
  let dt             = 0.0;        
  // シミュレータ
  let mut target = DogSimulation::new();
  target.set_random_seed(134)
        .set_measurement_var(
           voltage_std.powf(2.0)
         )
        .set_x(actual_voltage);
  // フィルタ
  let mut obfilter = ObjObsFilter::new(
    target,
    filter_fn
  );
  obfilter.set_process_model(
             (0.0, process_var)
           )
          .set_sensor_var(
             voltage_std.powf(2.0)
           )
          .set_x(x)
          .set_dt(dt);
  // 繰返し観測
  let mut prex_mean = x.0;
  for i in 0..10 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    let z = obfilter.zs(); // ノイズ有観測値
                           // 事前予測値,分散
    let (p_mean, p_var) = obfilter.prior();
                           // 事後分布
    let (x_mean, _x_var) = obfilter.xs();
    
    // 予測値表示
    lp.set_data(i as f32, p_mean)
      .draw(&mut display)
      .unwrap();
    lp.set_data((i + 1) as f32, p_mean)
      .draw(&mut display)
      .unwrap();
    lp.reset_data();
    // 観測値表示
    es.mode_fillrectangle()
      .set_shape_color(Z_COLOR)
      .set_data((i + 1) as f32, z)
      .draw(&mut display)
      .unwrap();
    delay.delay_ms(100 as u16);
    // 残差表示
    lz.set_data((i + 1) as f32, p_mean)
      .draw(&mut display)
      .unwrap();
    lz.set_data((i + 1) as f32, z)
      .draw(&mut display)
      .unwrap();
    lz.reset_data();
    delay.delay_ms(100 as u16);
    // フィルタ結果表示
    lf.set_data(i as f32, prex_mean)
      .draw(&mut display)
      .unwrap();
    lf.set_data((i + 1) as f32, x_mean)
      .draw(&mut display)
      .unwrap();
    lf.reset_data();
    // ガウスグラフ表示
    e2.mode_clear()
      .draw(&mut display)
      .unwrap();
    eg.set_data(p_mean, p_var)
      .draw(&mut display)
      .unwrap();
    // 前回予測値
    prex_mean = x_mean;
    // ウエイト
    delay.delay_ms(100 as u16);
  }
  // 終了
  loop {}
}
// ----------------------------------------
// ----------------------------------------
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 120_u32),  // 表示サイズ
    0..10,               // X目盛レンジ
    100..250,            // Y目盛レンジ
    (1.0, 10.0),        // 補正率(x,y)
    (2, 50),             // 目盛刻み
                         // タイトル
    "",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
// ガウス用グラフセッティング
fn bar2graph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 121),            // 表示開始位置
    (320_u32, 119_u32),  // 表示サイズ
    10..250,             // X目盛レンジ
    0..100,              // Y目盛レンジ
    (10.0, 100.0),       // 補正率(x,y)
    (20, 25),            // 目盛刻み
                         // タイトル
    "",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}