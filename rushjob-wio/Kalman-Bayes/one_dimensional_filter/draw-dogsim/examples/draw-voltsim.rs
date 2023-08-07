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
use emb_shapegraph::*;
use emb_linegraph::*;
use one_dimensional::*;
use dogsimulation::DogSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const DOT_COLOR  : Rgb565 = Rgb565::WHITE;
const CIRC_COLOR : Rgb565 = Rgb565::WHITE;

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
  // 線グラフ設定
  let mut e2 = bar2graph_setting();
  let mut lv = EmbLinegraph::new(&e2);
               lv.set_shape_color(LINE_COLOR);
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
  // 線グラフ表示領域クリア
  e2.mode_clear().draw(&mut display).unwrap();
  e2.mode_scale().draw(&mut display).unwrap();
// ----------------------------------------
// ----------------------------------------
  // フィルタ関数
//  // 1.ガウス分布の積:ベイズ的アプローチ
//  let filter_fn = OneDimKalman; 
  // 2.残差カルマンゲイン形式
  let filter_fn = OneDimKalmanRest;
  // 設定値
  let process_var    = 0.05.powf(2.0); 
  let voltage_std    = 0.13;
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
  for i in 0..50 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    let z = obfilter.zs(); // ノイズ有観測値
                           // 事後予測値,分散
    let (x_mean, x_var) = obfilter.xs();
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
    // 事後分布分散表示
    let stdv = x_var.sqrt();
      // 上
    lu.set_data(i as f32, x_mean + stdv)
      .draw(&mut display)
      .unwrap();
      // 下
    ld.set_data(i as f32, x_mean - stdv)
      .draw(&mut display)
      .unwrap();
    // フィルタ結果表示
    lv.set_data(i as f32, x_var)
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
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 120_u32),  // 表示サイズ
    0..50,               // X目盛レンジ
    1600..1700,          // Y目盛レンジ
    (1.0, 100.0),        // 補正率(x,y)
    (10, 20),           // 目盛刻み
                         // タイトル
    ":Filter,:Measurements",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
// グラフセッティング2
fn bar2graph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 121),            // 表示開始位置
    (320_u32, 119_u32),  // 表示サイズ
    0..50,               // X目盛レンジ
    4..17,               // Y目盛レンジ
    (1.0, 1000.0),       // 補正率(x,y)
    (10, 2),             // 目盛刻み
                         // タイトル
    ":Variance",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
