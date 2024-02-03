
use us_kalmanfilter::*;
use discrete_white_noise::*;
use robot2d::Robot2d;

// 次元設定
type M = U4; // 状態、プロセスモデル
type N = U2; // 観測値
type C = U1; // 制御入力
type B = U2; // プロセスノイズブロック
type G = U9; // シグマ点数
type LR= U1; 
type LC= U1;

fn main() {
// ----------------------------------------
  // 設定
  let dt = 1.;
  let r  = 0.3 * 0.3;
  let q  = 0.02;
//  let p = OMatrix::<f32, M, M>::new(
//    130.0, 0.1,
//    0.1, 90.0
//  );
// ----------------------------------------
  // シミュレータ
  let mut target = Robot2d::new();
// ----------------------------------------
  // シグマポイント
  let sg = <MSSigmaPoints<M, G>>::new(
    0.1, // alpha
    2.0, // beta
    1.0, // kappa
  );

  // 無香料変換(状態)
  let utx = <UsTransform<M, G>>::new();
  // 無香料変換(観測)
  let utz = <UsTransform<N, G>>::new();

  // フィルタ
  let mut kf = 
    <UsKalmanFilter<M, N, C, G, LR, LC>>
       ::new(sg, utx, utz);

  kf.x.copy_from_slice(&[0., 0., 0., 0.]);
  kf.F.copy_from_slice(
    &[1., 0., 0., 0.,
      dt, 1., 0., 0.,
      0., 0., 1., 0.,
      0., 0., dt, 1.0]
  );
  kf.H.copy_from_slice(
    &[1., 0.,
      0., 0.,
      0., 1.,
      0., 0.]
  );
  kf.R *= r;
//  kf.P = p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
  
  let (qd, bd) = (M::dim(), B::dim());
  (0..qd/bd).for_each(|i| 
    kf.Q.view_mut((i*bd, i*bd), (bd, bd))
      .copy_from(&bn)
  );

    println!("Q:{}", &kf.Q.transpose());

  // 繰返し観測
  for _ in 0..10 {
    let z = target.read();
    kf.z.copy_from_slice(&z);
    println!("z:{}", &kf.z);
    // フィルタ実行
    kf.predict();
    // 途中文字列表示
    println!("x:{}",&kf.x);
    println!("p:{}", &kf.P.transpose());
    kf.update();
    // 途中文字列表示
    println!("x:{}",&kf.x);
    println!("p:{}", &kf.P.transpose());
  }
}
// ----------------------------------------
// ----------------------------------------

