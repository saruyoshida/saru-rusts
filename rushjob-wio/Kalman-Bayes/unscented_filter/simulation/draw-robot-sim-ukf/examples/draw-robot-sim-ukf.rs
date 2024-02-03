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
use us_kalmanfilter::*;
use discrete_white_noise::*;
use robot2d::Robot2d;

use emb_bargraph::*;
use emb_linegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const L2_COLOR   : Rgb565 = Rgb565::YELLOW;

// カルマンフィルタ次元設定
type M = U4; // 状態、プロセスモデル
type N = U2; // 観測値
type C = U1; // 制御入力
type B = U2; // プロセスノイズブロック
type G = U9; // シグマ点数
type LR= U1; // 未使用
type LC= U1; // 未使用

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
  // 線形ライン表示設定
  let mut lm = EmbLinegraph::new(&eb);
               lm.set_shape_color(L2_COLOR)
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
  // グラフ領域クリア、目盛表示
  eb.mode_clear().draw(&mut display).unwrap();
  eb.mode_scale().draw(&mut display).unwrap();
// ========================================
// シミュレーション設定
  let dt = 1.0;
  let r  = 0.09;
  let q  = 0.02;
// シミュレータ
  let mut r2 = Robot2d::new();
  r2.set_random_seed(234);
  r2.noise_std = r;
// ========================================
// 線形カルマンフィルタ設定
  // フィルタ
  let mut kf = <KalmanFilter<M, N, C>>::new();
  kf.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 0.0,
      dt , 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, dt , 1.0]
  );
  kf.H.copy_from_slice(
    &[1.0, 0.0,
      0.0, 0.0,
      0.0, 1.0,
      0.0, 0.0]
  );
  kf.R *= r;
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
// 無香料カルマンフィルタ設定
  // シグマポイント
  let sg = <MSSigmaPoints<M, G>>::new(
    0.1, // alpha
    2.0, // beta
   -1.0, // kappa
  );
  // 無香料変換(状態)
  let utx = <UsTransform<M, G>>::new();
  // 無香料変換(観測)
  let utz = <UsTransform<N, G>>::new();
  // フィルタ
  let mut ukf = <UsKalmanFilter
                 <M, N, C, G, LR, LC>
                >::new(sg, utx, utz);
  ukf.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 0.0,
      dt , 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, dt , 1.0]
  );
  ukf.H.copy_from_slice(
    &[1.0, 0.0,
      0.0, 0.0,
      0.0, 1.0,
      0.0, 0.0]
  );
  kf.R *= r; 
  // プロセスノイズ設定
  let (qd, bd) = (M::dim(), B::dim());
  (0..qd/bd).for_each(|i| 
    kf.Q.view_mut((i*bd, i*bd), (bd, bd))
      .copy_from(&bn)
  );
// ========================================
// 線形カルマンフィルタ
  for _ in 0..100 {
    let z = r2.read();
    kf.z.copy_from_slice(&z);
    kf.predict();
    kf.update();
// グラフ表示
    lm.set_data(kf.x[(0, 0)], kf.x[(2, 0)])
      .draw(&mut display)
      .unwrap();
  }
// ========================================
// リセット
  r2.pos = (0., 0.);
  lm.reset_data().set_shape_color(LINE_COLOR)
                 .set_shape_width(1);
// ========================================
// 無香料カルマンフィルタ
  for _ in 0..100 {
    let z = r2.read();
    ukf.z.copy_from_slice(&z);
    ukf.predict();
    ukf.update();
// グラフ表示
    lm.set_data(ukf.x[(0, 0)], ukf.x[(2, 0)])
      .draw(&mut display)
      .unwrap();
  }
  // 終了
  loop {}
}
// ========================================
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 240_u32),  // 表示サイズ
    0..1000,              // X目盛レンジ
    0..1000,             // Y目盛レンジ
    (10.0, 10.0),        // 補正率(x,y)
    (200, 200),            // 目盛刻み
                         // タイトル
    "use UKF",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
