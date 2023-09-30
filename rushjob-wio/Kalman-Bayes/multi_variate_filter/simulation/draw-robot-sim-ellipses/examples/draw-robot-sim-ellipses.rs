#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use mv_kalmanfilter::*;
use discrete_white_noise::*;
use robot2d::Robot2d;

use emb_bargraph::*;
use emb_covargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const ELLP_COLOR : Rgb565 = Rgb565::YELLOW;
const SHAPE_COLOR: Rgb565 = Rgb565::RED;

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
  // 共分散グラフ表示設定
  let mut es = EmbCovargraph::new(&eb);
               es.set_shape_color(ELLP_COLOR)
                 .set_std(&[3.0]);
  // フィルタライン表示設定
  let mut lm = EmbLinegraph::new(&eb);
               lm.set_shape_color(LINE_COLOR)
                 .set_shape_width(2);
  // 観測値表示設定
  let mut sz = EmbShapegraph::new(&eb);
               sz.set_shape_color(SHAPE_COLOR)
                 .set_shape_width(2)
                 .mode_circle();
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
// ========================================
// シミュレーション設定
  let dt = 1.0;
  let r  = 0.35;
  let q  = 0.04;
  let p  = 500.0;
// ----------------------------------------
// シミュレータ
  let mut r2 = Robot2d::new();
  r2.pos = (0.0, 0.0);
  r2.vel = (2.0, 0.2);
  r2.noise_std = r;
// ----------------------------------------
// カルマンフィルタ設定
  // 次元設定
  type M = U4; // 状態、プロセスモデル
  type N = U2; // 観測値
  type C = U1; // 制御入力
  type B = U2; // プロセスノイズブロック
  // フィルタ
  let mut kf = <KalmanFilter<M, N, C>>::new();
  kf.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 0.0,
      dt , 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, dt , 1.0]
  );
  kf.H.copy_from_slice(
    &[1.0/0.3048, 0.0,
      0.0       , 0.0,
      0.0       , 1.0/0.3048,
      0.0       , 0.0]
  );
  kf.R *= r*r;
  kf.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q*q); 
    // プロセスノイズ設定
    // block_diag的なことをする
  let (qd, bd) = (M::dim(), B::dim());
  (0..qd/bd).for_each(|i| 
    kf.Q.view_mut((i*bd, i*bd), (bd, bd))
      .copy_from(&bn)
  );
// ========================================
// 繰返し観測
  for _ in 0..30 {
// ========================================
    // 観測値取得
    let z = r2.read();
    kf.z.copy_from_slice(&z);
    // フィルタ実行
    kf.predict();
    kf.update();
// ========================================
// グラフ表示
    // 共分散円表示
    es.set_data(
      &[kf.x[(0, 0)], kf.x[(2, 0)]],
      &[kf.P[(0, 0)], kf.P[(0, 2)],
        kf.P[(2, 0)], kf.P[(2, 2)]
       ]
    )
    .draw(&mut display)
    .unwrap();
    // 観測値表示
    sz.set_data(z[0]*0.3048, z[1]*0.3048)
      .draw(&mut display)
      .unwrap();
    // フィルタ結果表示
    lm.set_data(kf.x[(0, 0)], kf.x[(2, 0)])
      .draw(&mut display)
      .unwrap();
// ----------------------------------------
//    // ウエイト
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
    (320_u32, 160_u32),  // 表示サイズ
    0..200,              // X目盛レンジ
    -40..50,             // Y目盛レンジ
    (10.0, 10.0),        // 補正率(x,y)
    (25, 10),            // 目盛刻み
                         // タイトル
    "robot sim ellipses",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
