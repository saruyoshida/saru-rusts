#![no_std]
use filter_base::*;
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use nalgebra::SMatrix;
#[allow(unused_imports)]
use micromath::F32Ext;

// フィルタ供給配列数
pub const FLC: usize = 2;
// フィルタ供給種類
type Kf1 = KalmanFilter<2, 1, 1>;
type Kf2 = KalmanFilter<3, 1, 1>;
pub enum Filters {
  Kf1(Kf1), 
  Kf2(Kf2), 
}
#[allow(non_snake_case)]
impl Filters {
  // filter_supply_implマクロによる実装
  filter_supply_impl!(
    Kf1,
    Kf2,
  );
}
// 供給フィルタの格納 ====================
pub fn filter_supply() -> [Filters; FLC] { 
  [cv_filter(),
   ca_filter(),
  ]
}
// 定常速度フィルタ =======================
fn cv_filter() -> Filters {
// シミュレーション設定
  let dt     = 0.1;
  let r_std  = 0.2;
  let q      = 0.02;
  let p      = 3.0;
// カルマンフィルタ設定
  const M: usize = 2; // 状態、プロセスモデル
  const N: usize = 1; // 観測値
  const C: usize = 1; // 制御入力
  const B: usize = 2; // ﾌﾟﾛｾｽﾉｲｽﾞﾌﾞﾛｯｸ
  // フィルタ
  let mut kf = KalmanFilter::<M, N, C>::new();
  kf.fb.F.copy_from_slice(
    &[1.0, 0.0, 
      dt , 1.0,
     ]
  );
  kf.fb.H.copy_from_slice(&[1.0, 0.0]);
  kf.fb.R *= r_std*r_std;
  kf.fb.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
    // プロセスノイズ設定
  kf.fb.Q.copy_from(&bn);
  // 累積尤度計算設定
  kf.fb.cum_lh = Some(0.8);

  Filters::Kf1(kf)
}
// 定常加速度フィルタ =======================
fn ca_filter() -> Filters {
// シミュレーション設定
  let dt     = 0.1;
  let r_std  = 0.2;
  let q      = 0.02;
  let p      = 3.0;
// カルマンフィルタ設定
  const M: usize = 3; // 状態、プロセスモデル
  const N: usize = 1; // 観測値
  const C: usize = 1; // 制御入力
  const B: usize = 3; // ﾌﾟﾛｾｽﾉｲｽﾞﾌﾞﾛｯｸ
  // フィルタ
  let mut kf = KalmanFilter::<M, N, C>::new();
  kf.fb.F.copy_from_slice(
    &[1.0,       0.0, 0.0,
      dt ,       1.0, 0.0,
      0.5*dt*dt, dt,  1.0,
     ]
  );
  kf.fb.H.copy_from_slice(&[1.0, 0.0, 0.0]);
  kf.fb.R *= r_std;
  kf.fb.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
    // プロセスノイズ設定
  kf.fb.Q.copy_from(&bn);
  // 累積尤度計算設定
  kf.fb.cum_lh = Some(0.2);

  Filters::Kf2(kf)
}
