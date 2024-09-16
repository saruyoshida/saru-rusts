#![no_std]
use filter_base::*;
use mv_kalmanfilter::*;
use discrete_white_noise::*;
use nalgebra::SMatrix;
use ms_sigmapoints::*;
use us_transform::*;
use us_kalmanfilter::*;
#[allow(unused_imports)]
use micromath::F32Ext;

// フィルタ供給配列数
pub const FLC: usize = 6;
// フィルタ供給種類
type Kf1 = KalmanFilter<2, 1, 1>;
type Kf2 = KalmanFilter<3, 1, 1>;
type Kf3 = UsKalmanFilter<4, 2, 1, 9, 1, 1>;
pub enum Filters {
  Kf1(Kf1), 
  Kf2(Kf2),
  Kf3(Kf3),
}
#[allow(non_snake_case)]
impl Filters {
  // filter_supply_implマクロによる実装
  filter_supply_impl!(
    Kf1,
    Kf2,
    Kf3,
  );
}
// 供給フィルタの格納 ====================
pub fn filter_supply() -> [Filters; FLC] { 
  [ca_filter(),
   ca_filter_width_alpha(),
   cv_filter(),
   us_filter(),
   ca_filter_noise_eps(),
   ca_filter_noise_std(),
  ]
}
// 加速度フィルタそのまま
fn ca_filter() -> Filters {
  Filters::Kf2(mv_filter1())
}
// 加速度フィルタに減衰記憶を設定
fn ca_filter_width_alpha() -> Filters {
  let mut kf = mv_filter1();
  // 減衰記憶ﾊﾟﾗﾒｰﾀ設定
  kf.fb.alpha = Some(0.8);
  Filters::Kf2(kf)
}
// 加速度フィルタにepsでのﾌﾟﾛｾｽﾉｲｽﾞ調整を設定
fn ca_filter_noise_eps() -> Filters {
  let mut kf = mv_filter1();
  // ﾉｲｽﾞ調整ｶｳﾝﾀにゼロを設定
  kf.fb.ns_count = Some(0);
  Filters::Kf2(kf)
}
// 加速度フィルタにstdでのﾌﾟﾛｾｽﾉｲｽﾞ調整を設定
fn ca_filter_noise_std() -> Filters {
  let mut kf = mv_filter1();
  // ﾉｲｽﾞ調整ｶｳﾝﾀにゼロを設定
  kf.fb.ns_count = Some(0);
  // ﾉｲｽﾞ分散に値設定
  kf.fb.phi = Some(0.02);
  // ﾌﾟﾛｾｽﾉｲｽﾞｽｹｰﾘﾝｸﾞ係数設定
  kf.fb.Q_scale_factor = 1000.;
  // std_scale設定
  kf.fb.Q_adjust_param = 2.;
  // ﾌﾟﾛｾｽﾉｲｽﾞ調整関数入替え
  kf.fb.noise_adjust_fn = noise_adjast_std;
  Filters::Kf2(kf)
}
#[allow(non_snake_case)]
fn noise_adjast_std
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
  const B: usize = 3; // ﾌﾟﾛｾｽﾉｲｽﾞﾌﾞﾛｯｸ
  let std = S[(0, 0)].sqrt();
  let dt = 1.0;
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
// 加速度フィルタ ==========================
fn mv_filter1() -> Kf2 {
// シミュレーション設定
  let dt     = 1.0;
  let r_std  = 6.0;
  let q      = 0.02;
  let p      = 100.0;
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
      0.5*dt*dt, dt , 1.0
     ]
  );
  kf.fb.H.copy_from_slice(&[1.0, 0.0, 0.0]);
  kf.fb.R *= r_std*r_std;
  kf.fb.P.set_partial_diagonal(
    [p, 1.0, 1.0].into_iter()
  );
  // 累積尤度計算設定
  kf.fb.cum_lh = Some(0.4);
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
    // プロセスノイズ設定
  kf.fb.Q.copy_from(&bn);
  kf
}

// 定常速度フィルタ =======================
fn cv_filter() -> Filters {
// シミュレーション設定
  let vl = 1.0;
  let r  = 5.0;
  let q  = 0.02;
  let p  = 50.0;
//  let dt = 1.0;
// カルマンフィルタ設定
  // 次元設定
  const M: usize = 2; // 状態、プロセスモデル
  const N: usize = 1; // 観測値
  const C: usize = 1; // 制御入力
  // フィルタ
  let mut kf = KalmanFilter::<M, N, C>::new();
  kf.fb.x.copy_from_slice(&[0.0, vl]);
  kf.fb.F.copy_from_slice(
    &[1.0, 0.0, vl, 1.0]
  );
  kf.fb.H.copy_from_slice(&[1.0, 0.0]);
  kf.fb.R *= r;
  kf.fb.P *= p;
  // ノイズ設定 
    // プロセスノイズ設定
  kf.fb.Q *= q;
  // 累積尤度計算設定
  kf.fb.cum_lh = Some(0.2);

  Filters::Kf1(kf)
}
// ========================================
// 無香料カルマンフィルタ設定
fn us_filter() -> Filters {
  const M: usize = 4; // 状態、プロセスモデル
  const N: usize = 2; // 観測値
  const C: usize = 1; // 制御入力
  const B: usize = 2; // ノイズブロック
  const G: usize = 9; // シグマ点数
  const LR: usize= 1; // 未使用
  const LC: usize= 1; // 未使用
  // シミュレーション設定
  let dt = 1.0;
  let r  = 0.09;
  let q  = 0.02;
  // シグマポイント
  let sg = MSSigmaPoints::<M, G>::new(
    0.1, // alpha
    2.0, // beta
   -1.0, // kappa
  );
  // 無香料変換(状態)
  let utx = UsTransform::<M, G>::new();
  // 無香料変換(観測)
  let utz = UsTransform::<N, G>::new();
  // フィルタ
  let mut ukf = UsKalmanFilter
                ::<M, N, C, G, LR, LC>
                ::new(sg, utx, utz);
  ukf.fb.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 0.0,
      dt , 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, dt , 1.0]
  );
  ukf.fb.H.copy_from_slice(
    &[1.0, 0.0,
      0.0, 0.0,
      0.0, 1.0,
      0.0, 0.0]
  );
  ukf.fb.R *= r; 
  // ノイズ設定
    // ノイズブロック作成
  let bn: SMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q*q); 
  // プロセスノイズ設定
  (0..M/B).for_each(|i| 
    ukf.fb.Q.view_mut((i*B, i*B), (B, B))
       .copy_from(&bn)
  );
  // 累積尤度計算設定
  ukf.fb.cum_lh = Some(0.4);

  Filters::Kf3(ukf)
}
// ========================================
