#![no_std]
#![no_main]

use panic_halt as _;
use core::f32::consts::PI;
use wio_terminal as wio;
use micromath::F32Ext;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use us_kalmanfilter::*;
use discrete_white_noise::*;
use radaraccsim::*;

use emb_bargraph::*;
use emb_linegraph::*;

// 表示色設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const LINE_COLOR : Rgb565 = Rgb565::YELLOW;
// ========================================
// カルマンフィルタ次元設定
type M = U3; // 状態、プロセスモデル
type N = U2; // 観測値
type C = U1; // 制御入力
type B = U2; // プロセスノイズブロック
type G = U7; // シグマ点数 Mx2+1
type LR= U1; // 未使用
type LC= U1; // 未使用
// シミュレータ設定
const RADAR_POS:[f32; 2] = [0., 0.];
// ========================================
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
  // 位置グラフ
  let mut gp = graph_setting_pos();
  let mut lp = EmbLinegraph::new(&gp);
               lp.set_shape_color(LINE_COLOR);
  // 速度グラフ
  let mut gv = graph_setting_vel();
  let mut lv = EmbLinegraph::new(&gv);
               lv.set_shape_color(LINE_COLOR);
  // 高度グラフ
  let mut ga = graph_setting_alt();
  let mut la = EmbLinegraph::new(&ga);
               la.set_shape_color(LINE_COLOR);
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
  gp.mode_scale().draw(&mut display).unwrap();
  gv.mode_scale().draw(&mut display).unwrap();
  ga.mode_scale().draw(&mut display).unwrap();
// ========================================
// シミュレーション設定
  let dt = 3.0;
  let range_std = 5.;
  let elevation_angle_std = 0.5/180. * PI;
// シミュレータ
  // レーダー
  let mut radar = RadarStation::new();
  radar.pos = RADAR_POS;
  radar.range_std = range_std;
  radar.elev_angle_std = elevation_angle_std;
  // 航空機
  let mut ac = ACSim::new();
  ac.pos = [0.  , 1000.];
  ac.vel = [100.,    0.];
  ac.vel_std = 0.02;
  ac.dt  = dt;
// ========================================
// 無香料カルマンフィルタ設定
  // シグマポイント
  let sg = <MSSigmaPoints<M, G>>::new(
    0.1, // alpha
    2.0, // beta
    0.0, // kappa
  );
  // 無香料変換(状態)
  let utx = <UsTransform<M, G>>::new();
  // 無香料変換(観測)
  let utz = <UsTransform<N, G>>::new();
  // 無香料フィルタ
  let mut ukf = 
    <UsKalmanFilter<M, N, C, G, LR, LC>>
    ::new(sg, utx, utz);
  // 状態関数
  ukf.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 
      dt , 1.0, 0.0,
      0.0, 0.0, 1.0]
  );
  // 観測関数
  ukf.hx = h_radar;
  // 観測値ノイズ
  ukf.R.set_partial_diagonal(
    [range_std.powi(2),
     elevation_angle_std.powi(2)]
    .into_iter()
  );
  // 状態変数
  ukf.x.copy_from_slice(&[0., 90., 1100.]);
  // 状態共分散
  ukf.P.set_partial_diagonal(
    [300.0.powi(2), 
     30.0.powi(2), 
     150.0.powi(2)
    ].into_iter()
  );
  // プロセスノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, 0.1); 
    // プロセスノイズ設定
  let bd = B::dim();
  ukf.Q.view_mut((0, 0), (bd, bd))
       .copy_from(&bn);
  ukf.Q[(bd, bd)] = 0.1;
// ========================================
  // 繰返し観測
  for i in (0..360 + dt as usize)
           .step_by(dt as usize) {
    ac.update();
    ukf.z.copy_from_slice(
      &(radar.noisy_reading(ac.pos))
    );
    ukf.predict();
    ukf.update();
// グラフ表示
    // 位置
    lp.set_data(i as f32, ukf.x[(0, 0)])
      .draw(&mut display)
      .unwrap();
    // 速度
    lv.set_data(i as f32, ukf.x[(1, 0)])
      .draw(&mut display)
      .unwrap();
    // 高度
    la.set_data(i as f32, ukf.x[(2, 0)])
      .draw(&mut display)
      .unwrap();
  }
  // 終了
  loop{}
}
// ========================================
// 無香料カルマンフィルタ関数定義
// 観測関数
#[allow(non_snake_case)]
fn h_radar<M, N, LR, LC>(
  x:  &OMatrix<f32, M, U1>,
  _H: &OMatrix<f32, N, M>,
  _zt:&OMatrix<f32, LR, LC>,
) -> OMatrix<f32, N, U1>
where
  M : DimName,
  N : DimName,
  LR: DimName,
  LC: DimName,
  DefaultAllocator: 
    Allocator<f32, M, U1> +
    Allocator<f32, N, M>  +
    Allocator<f32, N, U1> +
    Allocator<f32, LR, LC> 
{
  // 予測値の位置と高度を、観測値の直距離と
  // 仰角に変換する

  // レーダーからの距離
  let dx = x[(0, 0)] - RADAR_POS[0]; // 位置
  let dy = x[(2, 0)] - RADAR_POS[1]; // 高度
  // 直距離
  let slant_range = (dx.powi(2) + dy.powi(2))
                    .sqrt();
  // 仰角
  let elevation_angle = dy.atan2(dx);
  // 予測値を観測値形式に変換した行列を返却
  let mut h = OMatrix::<f32, N, U1>::zeros();
  h.copy_from_slice(
    &[slant_range, elevation_angle]
  );
  h
}
// ========================================
// グラフセッティング
  // 位置
fn graph_setting_pos() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 0),              // 表示開始位置
    (320_u32, 70_u32),   // 表示サイズ
    0..400,              // X目盛レンジ
    0..40000,            // Y目盛レンジ
    (1.0, 1.0),          // 補正率(x,y)
    (50, 10000),          // 目盛刻み
                         // タイトル
    "position(m)",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
  // 速度
fn graph_setting_vel() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 80),             // 表示開始位置
    (320_u32, 70_u32),   // 表示サイズ
    0..400,              // X目盛レンジ
    980..1010,           // Y目盛レンジ
    (1.0, 10.0),         // 補正率(x,y)
    (50, 10),            // 目盛刻み
                         // タイトル
    "velosity",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
  // 高度
fn graph_setting_alt() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    (0, 160),           // 表示開始位置
    (320_u32, 70_u32),  // 表示サイズ
    0..400,             // X目盛レンジ
    9500..10500,        // Y目盛レンジ
    (1.0, 10.0),        // 補正率(x,y)
    (50, 250),          // 目盛刻み
                        // タイトル
    "altitude",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
