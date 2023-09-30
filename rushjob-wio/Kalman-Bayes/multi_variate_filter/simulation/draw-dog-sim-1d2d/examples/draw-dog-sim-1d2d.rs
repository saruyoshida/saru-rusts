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
use dogsimulation::DogSimulation;
use one_dimensional::OneDimSimulation;

use emb_bargraph::*;
use emb_linegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::BLUE;
const DOT_COLOR  : Rgb565 = Rgb565::RED;
const TRC_COLOR  : Rgb565 = Rgb565::YELLOW;

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
  // 1Dフィルタライン表示設定
  let mut l1 = EmbLinegraph::new(&eb);
               l1.set_shape_color(LINE_COLOR);
  // 2Dフィルタライン表示設定
  let mut l2 = l1.clone();
               l2.set_shape_color(DOT_COLOR)
                 .mode_dotline();
  // 位置表示設定
  let mut lt = l1.clone();
               lt.set_shape_color(TRC_COLOR);
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
  //-- (1)(2) 
  let vl = 1.0;
//  //-- (3) 速度が制御入力uと異なる場合
//  let vl = -2.0;
  // --
  let r  = 5.0;
  let q  = 0.02;
  let p  = 50.0;
  let dt = 1.0;
// ----------------------------------------
// シミュレータ
  let mut target = DogSimulation::new();
  target.set_process_var(0.0)                 
        .set_measurement_var(r)
        .set_x(0.0)
        .set_velocity(vl);
// ----------------------------------------
// カルマンフィルタ設定
  // 2D -----------------------------------
  // 次元設定
  type M = U2; // 状態、プロセスモデル
  type N = U1; // 観測値
  type C = U1; // 制御入力
  type B = U2; // プロセスノイズブロック
  // フィルタ
  let mut kf = <KalmanFilter<M, N, C>>::new();
  kf.x.copy_from_slice(&[0.0, vl]);
  kf.F.copy_from_slice(&[1.0, 0.0, vl, 1.0]);
  kf.H.copy_from_slice(&[1.0, 0.0]);
  kf.R *= r;
  kf.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(vl, 0.0); 
    // プロセスノイズ設定
  kf.Q.copy_from(&bn);
  // 1D -----------------------------------
  // フィルタ
  let mut k1 = <KalmanFilter<N, N, N>>::new();
  k1.x.copy_from_slice(&[0.0]);
  k1.F.copy_from_slice(&[1.0]);
  k1.H.copy_from_slice(&[1.0]);
  k1.B.copy_from_slice(&[1.0]);
  k1.R *= r;
  k1.P *= p;
  // プロセスノイズ設定
  k1.Q *= q;
//  //-- (2)(3) u:制御入力を設定する場合
//  k1.u.copy_from_slice(&[1.0]);
  //--
// ========================================
// 繰返し観測
  for i in 0..100 {
// ========================================
    // 観測値取得
    let (tr, z) = target.move_and_sense(dt);
    kf.z.copy_from_slice(&[z]);
    k1.z.copy_from_slice(&[z]);
    // フィルタ実行
    kf.predict();
    k1.predict();
    kf.update();
    k1.update();
// ========================================
// グラフ表示
    // 位置表示
    lt.set_data(i as f32, tr)
      .draw(&mut display)
      .unwrap();
    // 2Dフィルタ結果表示
    l2.set_data(i as f32, kf.x[(0, 0)])
      .draw(&mut display)
      .unwrap();
    // 1Dフィルタ結果表示
    l1.set_data(i as f32, k1.x[(0, 0)])
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
// ----------------------------------------
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 240_u32),  // 表示サイズ
    0..100,              // X目盛レンジ
    //-- (1)(2)
    0..100,              // Y目盛レンジ
//    //-- (3)
//    -200..0,             // Y目盛レンジ
    //--
    (1.0, 1.0),          // 補正率(x,y)
    (20, 20),            // 目盛刻み
                         // タイトル
    "1d 2d comp",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
