#![no_std]
use filter_base::*;
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use nalgebra::SMatrix;
#[allow(unused_imports)]
use micromath::F32Ext;

// フィルタ供給配列数
pub const FLC: usize = 4;
// フィルタ供給種類
type Kf1 = KalmanFilter<2, 1, 1>;
pub enum Filters {
  Kf1(Kf1), 
}
#[allow(non_snake_case)]
impl Filters {
  // filter_supply_implマクロによる実装
  filter_supply_impl!(
    Kf1,
  );
}
// 供給フィルタの格納 ====================
pub fn filter_supply() -> [Filters; FLC] { 
  [cv_filter(),
   cv_filter_qvar20(),
   cv_filter_alpha102(),
   cv_filter_alpha105(),
  ]
}
// 定常速度フィルタそのまま
fn cv_filter() -> Filters {
  Filters::Kf1(mv_filter1())
}
// 定常速度フィルタのQのvarを20に変更
fn cv_filter_qvar20() -> Filters {
  let mut kf = mv_filter1();
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, 2, 2> =  
    DiscreteWhiteNoise::noise_block(0.1, 20.); 
    // プロセスノイズ設定
  kf.fb.Q.copy_from(&bn);
  Filters::Kf1(kf)
}
// 定常速度フィルタのalphaに1.02を設定
fn cv_filter_alpha102() -> Filters {
  let mut kf = mv_filter1();
  kf.fb.alpha = Some(1.02);
  Filters::Kf1(kf)
}
// 定常速度フィルタのalphaに1.05を設定
fn cv_filter_alpha105() -> Filters {
  let mut kf = mv_filter1();
  kf.fb.alpha = Some(1.05);
  Filters::Kf1(kf)
}
// 定常速度フィルタ =======================
fn mv_filter1() -> Kf1 {
// シミュレーション設定
  let dt     = 0.1;
  let r_std  = 1.2;
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
  kf
}
