
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use dogsimulation::DogSimulation;
use one_dimensional::OneDimSimulation;

fn main() {
// ----------------------------------------
  // 設定
  let dt = 1.0;
  let x = &[0.0, 0.0];
  let f = &[1.0, 0.0, dt, 1.0];
  let h = &[1.0, 0.0];
  let r  = 5.0;
  let q  = 0.2;
  let p  = 20.0;
// ----------------------------------------
  // シミュレータ
  let mut target = DogSimulation::new();
  target.set_random_seed(134)
        .set_process_var(q)                     
        .set_measurement_var(r)
        .set_x(0.0)
        .set_velocity(dt);
// ----------------------------------------
  // 次元設定
  type M = U2; // 状態、プロセスモデル
  type N = U1; // 観測値
  type C = U1; // 制御入力
  type B = U2; // プロセスノイズブロック
  // フィルタ
  let mut kf = <KalmanFilter<M, N, C>>::new();
  kf.x.copy_from_slice(x);
  kf.F.copy_from_slice(f);
  kf.H.copy_from_slice(h);
  kf.R *= r;
  kf.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
  kf.Q.copy_from(&bn);
//  // block_diag的なことをする
//  let (b, _) = bn.shape();
//  (0..b).for_each(|i| 
//    kf.Q.fixed_view_mut::<B, B>(i*b, i*b)
//      .copy_from(&bn)
//  );
  // 繰返し観測
  for _ in 0..10 {
    let (_, z) = target.move_and_sense(dt);
    kf.z.copy_from_slice(&[z]);
    // フィルタ実行
    kf.predict();
    kf.update();
    // 途中文字列表示
    println!(
      "x1:{:03.03}  x2:{:03.03}", 
      kf.x[(0, 0)], kf.x[(1, 0)]
    );
    println!("{}", kf.B * kf.u);
  }
}
// ----------------------------------------
// ----------------------------------------

