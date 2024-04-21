pub use embedded_graphics::{
  pixelcolor::Rgb565,
  prelude::*,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, 
    SimulatorDisplay, 
    SimulatorEvent, 
    Window,
};
pub use std::{thread, time::Duration};

use us_kalmanfilter::*;
use robotukfsim::*;
use robotukffn::RobotUkfFn;

use emb_bargraph::*;
use emb_covargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;
// ========================================
// グラフ表示設定
// 表示色設定
pub const BASE_COLOR : Rgb565 = Rgb565::BLACK;
pub const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
pub const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
pub const LINE_COLOR : Rgb565 =Rgb565::YELLOW;
pub const ELLP_COLOR : Rgb565 = Rgb565::CYAN;
pub const SHAPE_COLOR: Rgb565 = Rgb565::GREEN;
// ========================================
fn main() -> 
  Result<(), core::convert::Infallible>
{
  let mut display: SimulatorDisplay<Rgb565> 
    = SimulatorDisplay::new(
                          Size::new(320, 240)
                        );
  let output_settings = OutputSettingsBuilder
                        ::new().scale(2)
                               .build();
  let mut window = Window::new(
                     "PrticleFilter", 
                     &output_settings
                   );
// ----------------------------------------
// グラフ表示設定
  // グラフ設定
  let mut eb = bargraph_setting();
  // 共分散グラフ表示設定
  let mut es = EmbCovargraph::new(&eb);
               es.set_shape_color(ELLP_COLOR)
                 .set_std(&[6.0]);
  // 実際位置表示設定
  let mut lp = EmbLinegraph::new(&eb);
               lp.set_shape_color(LINE_COLOR);
  // ランドマーク表示設定
  let mut sl = EmbShapegraph::new(&eb);
               sl.set_shape_color(SHAPE_COLOR)
                 .set_shape_diameter(6)
                 .mode_fillrectangle();
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
// ========================================
// シミュレーション設定
  let mut rb = RobotUkfSim::new();
// ========================================
// ランドマーク描画
  rb.landmarks.into_iter().for_each(|p|
    sl.set_data(p[0], p[1])
      .draw(&mut display)
      .unwrap()
  );
// ========================================
// 無香料カルマンフィルタ設定
  // シグマポイント
  let mut sg = <MSSigmaPoints<M, G>>::new(
    0.01,    // alpha
    2.0,     // beta
    0.0,     // kappa
  );
  sg.subtract = RobotUkfFn::residual_x;
  // 無香料変換(状態)
  let mut utx = <UsTransform<M, G>>::new();
  utx.mean_fn     = RobotUkfFn::state_mean;
  utx.residual_fn = RobotUkfFn::residual_x;
  // 無香料変換(観測)
  let mut utz = <UsTransform<N, G>>::new();
  utz.mean_fn     = RobotUkfFn::z_mean;
  utz.residual_fn = RobotUkfFn::residual_h;
  // 無香料フィルタ
  let mut ukf = 
    <UsKalmanFilter<M, N, C, G, LR, LC>>
    ::new(sg, utx, utz);
  ukf.residual_x  = RobotUkfFn::residual_x;
  ukf.residual_z  = RobotUkfFn::residual_h;
  ukf.dt = DT;
  // 状態関数
  ukf.fx = RobotUkfFn::fx;
  // 観測関数
  ukf.hx = RobotUkfFn::hx;
  // ランドマーク
  rb.landmarks.into_iter().enumerate()
    .for_each(|(i, lm)|
       ukf.lm.row_mut(i).copy_from_slice(&lm)
     );  
  // 観測値ノイズ
  ukf.R.set_partial_diagonal(
    [rb.sigma_range.powi(2), 
     rb.sigma_bearing.powi(2)
    ].into_iter().cycle().take(N::dim())
  );
  // 状態変数: ロボットの初期状態を設定
  ukf.x.copy_from_slice(&rb.sim_pos);
  // 状態共分散初期値
  ukf.P.set_partial_diagonal(
    [0.1, 0.1, 0.05].into_iter()
  );
  // プロセスノイズ設定
  ukf.Q *= 0.0001;
// ========================================
  // グラフ表示ワーク行列
  let (mut gx, mut gp) = (
    ukf.x.clone(), ukf.P.clone()
  );
  // 繰返し観測
  let cmds = make_cmd(); // 制御入力
  for (i, u) in cmds.enumerate() {
// ----------------------------------------
    // ロボット移動
    rb.move_next(DT/STEP as f32, &u);
    // フィルタ更新間隔
    if i % STEP == 0 {
      // 制御入力取得
      ukf.u.copy_from_slice(&u);
      // 予測
      ukf.predict();
      // グラフ表示用値コピー
      gx.copy_from(&ukf.x);
      gp.copy_from(&ukf.P);
      // 観測値取得
      ukf.z.copy_from_slice(rb.z());
      // 更新
      ukf.update();
    }
// ----------------------------------------
  // グラフ表示
    // 実際位置表示
    lp.set_data(rb.sim_pos[0], rb.sim_pos[1])
      .draw(&mut display)
      .unwrap();
    // 共分散円表示
    if i % (STEP * ELLIPSE_STEP) == 0 {
      es.set_data(
        &[gx[(0, 0)], gx[(1, 0)]],
        &[gp[(0, 0)], gp[(0, 1)],
          gp[(1, 0)], gp[(1, 1)]
         ]
      )
      .draw(&mut display)
      .unwrap();
    }
    window.update(&display);
  }
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {
      break 'running Ok(());
    }
  }
}
// ========================================
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    DSP_ST,            // 表示開始位置
    DSP_SZ,            // 表示サイズ
    DSP_XRS..DSP_XRE,  // X目盛レンジ
    DSP_YRS..DSP_YRE,  // Y目盛レンジ
    DSP_HSP,           // 補正率(x,y)
    DSP_SN,            // 目盛刻み
                       // タイトル
    "robot ukf sim",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
