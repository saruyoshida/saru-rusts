#![no_std]
use filter_base::*;
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use nalgebra::SMatrix;
#[allow(unused_imports)]
use micromath::F32Ext;

// フィルタ供給配列数
pub const FLC: usize = 9;
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
  [Filters::Kf1(mv_filter1( 1000., 2.0)),
   Filters::Kf1(mv_filter1( 1000., 3.0)),
   Filters::Kf1(mv_filter1( 1000., 0.1)),
   Filters::Kf1(mv_filter1( 1000., 1.0)),
   Filters::Kf1(mv_filter1(    1., 2.0)),
   Filters::Kf1(mv_filter1(   10., 2.0)),
   Filters::Kf1(mv_filter1(  100., 2.0)),
   Filters::Kf1(mv_filter1( 1000., 2.0)),
   Filters::Kf1(mv_filter1(10000., 2.0)),
  ]
}
// 定常速度フィルタ =======================
#[allow(non_snake_case)]
fn mv_filter1(
  Q_sacle_factor: f32,
  std_scale     : f32
) -> Kf1 {
// シミュレーション設定
  let dt     = 0.1;
  let r_std  = 0.2;
  let p      = 0.1;
  let phi    = 0.02;
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
  kf.fb.R *= r_std;
  kf.fb.P *= p;
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, phi); 
    // プロセスノイズ設定
  kf.fb.Q.copy_from(&bn);
  // ﾉｲｽﾞ調整ｶｳﾝﾀにゼロを設定
  kf.fb.ns_count = Some(0);
  // ﾌﾟﾛｾｽﾉｲｽﾞ調整関数入替え
  kf.fb.noise_adjust_fn = noise_adjust_fn;
  // ﾌﾟﾛｾｽﾉｲｽﾞ調整設定
  kf.fb.phi = Some(phi);
  kf.fb.Q_scale_factor = Q_sacle_factor; 
  kf.fb.Q_adjust_param = std_scale;
  kf
}
// 関数定義型:ﾌﾟﾛｾｽﾉｲｽﾞ調整関数実装
#[allow(non_snake_case)]
fn noise_adjust_fn
   <const M: usize, const N: usize>
(
  Q: &mut SMatrix<f32, M, M>, // Q
  y:     &SMatrix<f32, N, 1>, // y
  S:     &SMatrix<f32, N, N>, // S
  phi     : &mut f32,         // phi
  ns_count: &mut usize,       // ns_count
  Q_scale_factor: f32,
  std_scale: f32,
)
{  
  const B: usize = 2; // ﾌﾟﾛｾｽﾉｲｽﾞﾌﾞﾛｯｸ
  let std = S[(0, 0)].sqrt();
  let dt = 0.1;
  if y[(0, 0)].abs() > std_scale*std {
    *phi += Q_scale_factor;
    *ns_count += 1;
  } else if *ns_count > 0 {
    *phi -= Q_scale_factor;
    *ns_count -= 1;
  }
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, *phi);
    // プロセスノイズ設定
  Q.copy_from_slice(bn.as_slice());
}
