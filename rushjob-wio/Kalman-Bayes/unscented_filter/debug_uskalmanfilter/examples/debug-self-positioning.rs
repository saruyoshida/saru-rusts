
use us_kalmanfilter::*;
use robotukfsim::*;
use robotukffn::RobotUkfFn;

// ========================================
fn main() {
// ----------------------------------------
// シミュレーション設定
  let mut rb = RobotUkfSim::new();
// ========================================
// 無香料カルマンフィルタ設定
  // シグマポイント
  let mut sg = <MSSigmaPoints<M, G>>::new(
    0.01, // alpha
    2.0,  // beta
    0.0,  // kappa
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
  // 状態共分散
  ukf.P.set_partial_diagonal(
    [0.1, 0.1, 0.05].into_iter()
  );
  // プロセスノイズ設定
  ukf.Q *= 0.0001;
// ========================================
   println!("lm:{}",ukf.lm.transpose());
   println!("R:{}",ukf.R.transpose());
   println!("x:{}",ukf.x);
   println!("P:{}",ukf.P.transpose());
// ========================================
  // グラフ表示ワーク行列
  let (mut gx, mut gp) = (
    ukf.x.clone(), ukf.P.clone()
  );
  // 繰返し観測
  let cmds = make_cmd();
  for (i, u) in cmds.enumerate() {
    // ロボット移動
    rb.move_next(DT/STEP as f32, &u);
    // フィルタ更新間隔
    if i % STEP == 0 {
      // 制御入力取得
      ukf.u.copy_from_slice(&u);
// ========================================
   println!("u:{}",ukf.u);
   println!("z:{:?}",rb.z());
   println!("pos:{:?}",rb.sim_pos);
// ========================================
      // 予測
      ukf.predict();
      // グラフ表示用値コピー
// ========================================
   println!("x:{}",gx);
   println!("P:{}",gp.transpose());
   println!("u:{}",ukf.u);
   println!("z:{:?}",rb.z());
// ========================================
      gx.copy_from(&ukf.x);
      gp.copy_from(&ukf.P);
      // 観測値取得
      ukf.z.copy_from_slice(rb.z());
      // 更新
   println!("main update");
      ukf.update();
    }
  // グラフ表示
    // 実際位置表示
    println!("pos:{:?}",rb.sim_pos);
    // 共分散円表示
    if i % (STEP * ELLIPSE_STEP) == 0 {
      println!("x:{}",gx);
      println!("P:{}",gp.transpose());
    }
  }
  // 終了
}
