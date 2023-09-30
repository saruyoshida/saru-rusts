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
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use constantacc::*;

use emb_bargraph::*;
use emb_shapegraph::*;
use emb_linegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const DOT_COLOR  : Rgb565 = Rgb565::YELLOW;

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
  // フィルタグラフ設定
  let mut eb = bargraph_setting();
      // フィルタライン
  let mut lf = EmbLinegraph::new(&eb);
               lf.set_shape_color(LINE_COLOR);
      // トラック
  let mut lt = lf.clone();
               lt.set_shape_color(SCALE_COLOR)
                 .mode_dotline();
      // 観測値
  let mut sz = EmbShapegraph::new(&eb);
               sz.set_shape_color(SCALE_COLOR)
                 .mode_circle();                 
  // 位置分散値グラフ設定
  let mut e2 = bar2graph_setting();
      // 残差グラフ
  let mut ly = EmbLinegraph::new(&e2);
               ly.set_shape_color(LINE_COLOR);
        // 標準偏差上
  let mut lu = ly.clone();
               lu.set_shape_color(DOT_COLOR);
      // 標準偏差下
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
  // 目盛表示
  eb.mode_scale().draw(&mut display).unwrap();
  e2.mode_scale().draw(&mut display).unwrap();
// ========================================
// シミュレーション設定
  let dt     = 1.0;
  let r_std  = 6.0;
  let q      = 0.02;
  let p      = 100.0;
// ----------------------------------------
// シミュレータ
  let mut ca = ConstantAcc::new();
  ca.noise_scale = q;
// ----------------------------------------
// カルマンフィルタ設定
  // 次元設定
  type M = U3; // 状態、プロセスモデル
  type N = U1; // 観測値
  type C = U1; // 制御入力
  type B = U3; // プロセスノイズブロック
  // フィルタ
  let mut kf = <KalmanFilter<M, N, C>>::new();
  kf.F.copy_from_slice(
    &[1.0,       0.0, 0.0,
      dt ,       1.0, 0.0,
      0.5*dt*dt, dt , 1.0
     ]
  );
  kf.H.copy_from_slice(&[1.0, 0.0, 0.0]);
  kf.R *= r_std*r_std;
  kf.P.set_partial_diagonal(
    [p, 1.0, 1.0].into_iter()
  );
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
    // プロセスノイズ設定
  kf.Q.copy_from(&bn);
// ========================================
// 繰返し観測
  for i in 0..80 {
// ========================================
    // 観測値取得
    let x = ca.read();
    let z = ca.sense(&x, r_std);
    kf.z.copy_from_slice(&[z]);
    // フィルタ実行
    kf.predict();
    kf.update();
// ========================================
// グラフ表示
  // フィルタグラフ表示
    // 観測値表示
    sz.set_data(i as f32, z)
      .draw(&mut display)
      .unwrap();
    // トラック表示
    lt.set_data(i as f32, x[0])
      .draw(&mut display)
      .unwrap();
    // フィルタ結果表示
    lf.set_data(i as f32, kf.x[(0, 0)])
      .draw(&mut display)
      .unwrap();
  // 残差グラフ表示
    // 事前予測分散表示
      // 分散値
    let stdv = kf.P[(0, 0)].sqrt();
      // 分散上表示
    lu.set_data(i as f32, stdv)
      .draw(&mut display)
      .unwrap();
      // 分散下表示
    ld.set_data(i as f32, -stdv)
      .draw(&mut display)
      .unwrap();

    // 残差グラフ表示
    ly.set_data(i as f32, x[0] - kf.x[(0, 0)])
      .draw(&mut display)
      .unwrap();
// ----------------------------------------
    // ウエイト
//    delay.delay_ms(500 as u16);
  }
  // 終了
  loop {}
}
// ----------------------------------------
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 119_u32),  // 表示サイズ
    0..80,               // X目盛レンジ
    0..400,              // Y目盛レンジ
    (1.0, 1.0),          // 補正率(x,y)
    (10, 50),            // 目盛刻み
                         // タイトル
    "position x time",
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
    (0, 120),            // 表示開始位置
    (320_u32, 120_u32),  // 表示サイズ
    0..80,               // X目盛レンジ
    -60..80,             // Y目盛レンジ
    (1.0, 10.0),         // 補正率(x,y)
    (10, 20),            // 目盛刻み
                         // タイトル
    "pos var",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}

