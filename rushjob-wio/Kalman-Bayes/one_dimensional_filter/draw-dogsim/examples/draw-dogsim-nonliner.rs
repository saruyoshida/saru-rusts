#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

//use micromath::F32Ext;

use emb_bargraph::*;
use emb_linegraph::*;
use one_dimensional::*;
use nonlinersimulation::NonlinerSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
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
  // フィルタライン表示設定
  let mut lm = EmbLinegraph::new(&eb);
               lm.set_shape_color(LINE_COLOR)
                 .set_shape_width(2);
  // 観測値ライン表示設定
  let mut lz = EmbLinegraph::new(&eb);
               lz.set_shape_color(DOT_COLOR)
                 .mode_dotline()
                 .set_shape_width(1);
  // 画面クリア
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();
  // グラフ領域クリア、目盛表示
  eb.mode_clear().draw(&mut display).unwrap();
  eb.mode_scale().draw(&mut display).unwrap();
// ----------------------------------------
// ----------------------------------------
  // フィルタ関数
  // 1.ガウス分布の積:ベイズ的アプローチ
//  let filter_fn = OneDimKalman; 
  // 2.残差カルマンゲイン形式
  let filter_fn = OneDimKalmanRest;

  // 設定値
  let process_var = 2.0; 
  let sensor_var  = 30.0; 
  let x  = (100.0, 500.0); 
  let velocity    = 1.0; 
  let dt          = 1.0;
  // シミュレータ
  let mut target = NonlinerSimulation::new();
  target.set_random_seed(134);
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
  for i in 0..100 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    let z = obfilter.zs(); // ノイズ有観測値
                           // 事後予測値
    let (x_mean, _x_var) = obfilter.xs();
    // フィルタ結果表示
    lm.set_data(i as f32, x_mean)
      .draw(&mut display)
      .unwrap();
    // 観測値表示
    lz.set_data(i as f32, z)
      .draw(&mut display)
      .unwrap();
    // ウエイト
//    delay.delay_ms(500 as u16);
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
    (320_u32, 240_u32),  // 表示サイズ
    0..100,              // X目盛レンジ
    -40..50,             // Y目盛レンジ
    (1.0, 10.0),         // 補正率(x,y)
    (20, 10),            // 目盛刻み
                         // タイトル
    "nonliner",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
