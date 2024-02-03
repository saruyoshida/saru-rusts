
use core::f32::consts::PI;
use micromath::F32Ext;
use rand_distr::{Normal, Distribution};
use rand::prelude::*;

use us_kalmanfilter::*;
use discrete_white_noise::*;

// ========================================
// カルマンフィルタ次元設定
type M = U4; // 状態、プロセスモデル
type N = U2; // 観測値
type C = U1; // 制御入力
type B = U2; // プロセスノイズブロック
type G = U9; // シグマ点数 Mx2+1
type LR= U1; // 未使用
type LC= U1; // 未使用
// シミュレータ設定:レーダー位置
const SA_POS: [f32; 2] = [-400., 0.];
const SB_POS: [f32; 2] = [ 400., 0.];
// ========================================
fn main() {
// ========================================
// シミュレーション設定
  let dt = 0.1;
//  let mut target_pos = [100., 200.];
  let mut target_pos = [0., 0.];
  let std_noise = 0.5/180. * PI;
  // 乱数
  let mut rng = StdRng::from_seed([123; 32]);
  let normal = Normal::new(0.0, 1.0).unwrap();
  let mut randn = || normal.sample(&mut rng);
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
  ukf.fx = fx_vor;
  // 観測関数
  ukf.hx = hx_vor;
  // 観測値ノイズ
  ukf.R *= std_noise.powi(2);
  // 状態変数
  ukf.x.copy_from_slice(
    &[target_pos[0],
      1.,
      target_pos[1],
      1.,
     ]
  );
  // 状態共分散
  ukf.P *= 1000.;
  // プロセスノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, 1.0); 
    // block diag
  let (qd, bd) = (M::dim(), B::dim());
  (0..qd/bd).for_each(|i| 
    ukf.Q.view_mut((i*bd, i*bd), (bd, bd))
       .copy_from(&bn)
  );
// ========================================
  // 繰返し観測
  for _ in 0..300 {
    // 航空機移動
    (0..2).for_each(|i| 
      target_pos[i] += 1. + randn() * 0.0001
    );
    // 測定値取得
    ukf.z.copy_from_slice(
      &(measurement(
          &SA_POS, 
          &SB_POS,
          &target_pos,
        )
       )
    );
    // 測定値にノイズ追加
    (0..N::dim()).for_each(|i|
      ukf.z[(i, 0)] += randn() * std_noise
    );

    ukf.predict();
    ukf.update();
// グラフ表示
    println!("x:{:?}",ukf.x);
    println!("z:{:?}",ukf.z);
  }
  // 終了
}
// ========================================
// 無香料カルマンフィルタ関数定義
// 状態遷移関数(fx)
#[allow(non_snake_case)]
fn fx_vor<M, C>(
  x:  &OMatrix<f32, M, U1>,
  _u:  &OMatrix<f32, C, U1>,
  _F:  &OMatrix<f32, M, M>,
  _B:  &OMatrix<f32, M, C>,
  dt:f32,
) -> OMatrix<f32, M, U1>
where
  M : DimName,
  C : DimName,
  DefaultAllocator: 
    Allocator<f32, M, U1> +
    Allocator<f32, C, U1> +
    Allocator<f32, M, M>  + 
    Allocator<f32, M, C> 
{
  let mut a = x.clone();
  a[(0, 0)] += a[(1, 0)] * dt;
  a[(2, 0)] += a[(3, 0)] * dt;
  a
}
// 観測関数
#[allow(non_snake_case)]
fn hx_vor<M, N, LR, LC>(
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
  let mut h = OMatrix::<f32, N, U1>::zeros();
  h.copy_from_slice(
    &(measurement(
        &SA_POS, 
        &SB_POS,
        &[x[(0, 0)], x[(2, 0)]],
      )
     )      
  );
  h
}
// ========================================
// 他関数
// 方位
fn bearing(
  sensor: &[f32], 
  target_pos: &[f32]
) -> f32
{
  (target_pos[1] - sensor[1]).atan2
  (target_pos[0] - sensor[0])
}
// 測定
fn measurement(
  a_pos: &[f32],
  b_pos: &[f32],
  pos  : &[f32]
) -> [f32; 2]
{
  [bearing(a_pos, pos),
   bearing(b_pos, pos),
  ]
}
// ========================================

