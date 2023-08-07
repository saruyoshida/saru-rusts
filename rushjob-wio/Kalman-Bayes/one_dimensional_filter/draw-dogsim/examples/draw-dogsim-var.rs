#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use emb_bargraph::*;
use emb_linegraph::*;
use one_dimensional::*;
use emb_gaussgraph::*;
use dogsimulation::DogSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::GREEN;
const DOT_COLOR  : Rgb565 = Rgb565::RED;

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
    .set_shape_color(Rgb565::BLUE);
  // フィルタライン表示設定
  let mut lf = EmbLinegraph::new(&eb);
               lf.set_shape_color(LINE_COLOR);
  // 観測値ライン表示設定
  let mut lz = lf.clone();
               lz.mode_dotline()
                 .set_shape_color(DOT_COLOR);
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
  let process_var = 0.2;  // 犬の動きの分散
  let sensor_var  = 2.0;  // センサーの分散
  let x  = (0.0, 400.0);  // 犬の初期推定位置
  let velocity    = 1.0;  // 単位移動量
  let dt          = 1.0;  // タイムステップ        
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
  for i in 0..20 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    let z = obfilter.zs(); // ノイズ有観測値
                           // 事後分布
    let (x_mean, x_var) = obfilter.xs();
    // 観測値表示
    lz.set_data(i as f32, z)
      .draw(&mut display)
      .unwrap();
    // フィルタ結果表示
    lf.set_data(i as f32, x_mean)
      .draw(&mut display)
      .unwrap();
    // ガウスグラフ表示
    e2.mode_clear()
      .draw(&mut display)
      .unwrap();
    eg.set_data(i as f32, x_var)
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
    (320_u32, 160_u32),  // 表示サイズ
    0..25,               // X目盛レンジ
    0..3000,             // Y目盛レンジ
    (1.0, 100.0),        // 補正率(x,y)
    (5, 500),            // 目盛刻み
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
    (0, 161),            // 表示開始位置
    (320_u32, 79_u32),   // 表示サイズ
    0..250,              // X目盛レンジ
    0..100,              // Y目盛レンジ
    (10.0, 100.0),       // 補正率(x,y)
    (50, 20),            // 目盛刻み
                         // タイトル
    "",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}