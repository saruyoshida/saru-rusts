#![no_std]
use filter_base::*;
use mv_kalmanfilter::*;
use nalgebra::SMatrix;

// フィルタ供給配列数
pub const FLC: usize = 2;
// フィルタ次数
pub const M  : usize = 6;
pub const N  : usize = 2;
pub const C  : usize = 1;
// フィルタ供給種類
type Kf1 = KalmanFilter<M, N, C>;
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
  [ca_filter_normal(),
   ca_filter_q_zero(),
  ]
}
// 定常加速度フィルタそのまま
fn ca_filter_normal() -> Filters {
  Filters::Kf1(ca_filter())
}
// 定常加速度フィルタのQをゼロ
fn ca_filter_q_zero() -> Filters {
  let mut kf = ca_filter();
  // ノイズ設定
  kf.fb.Q *= 0.;
  Filters::Kf1(kf)
}
// 定常加速度フィルタ =======================
fn ca_filter() -> Kf1 {
// シミュレーション設定
  let dt     = 1.0;
  let r_std  = 1.0;
  let p      = 1.0e-12;
// カルマンフィルタ設定
  const B: usize = 3; // ﾌﾞﾛｯｸ
  // フィルタ
  let mut kf = Kf1::new();
  let bf = SMatrix::<f32, B, B>
                  ::from_column_slice (
    &[1.0,       0.0, 0.0,
      dt ,       1.0, 0.0,
      0.5*dt*dt, dt,  1.0,
     ]
  );
  (0..M/B).for_each(|i| 
    kf.fb.F.view_mut((i*B, i*B), (B, B))
      .copy_from(&bf)
  );
  kf.fb.x.copy_from_slice(
    &[2000., 0., 0., 10000., -15., 0.]
  );
  kf.fb.P *= p;
  kf.fb.R *= r_std*r_std;
  let mut bn = SMatrix::<f32, B, B> 
                      ::from_column_slice (
    &[0.05 , 0.125, 1./6.,
      0.125, 1./3.,   0.5,
      1./5., 0.5  ,   1.0
     ]
  );
  bn *= 1.0e-3;
  (0..M/B).for_each(|i| 
    kf.fb.Q.view_mut((i*B, i*B), (B, B))
       .copy_from(&bn)
  );
  kf.fb.H.copy_from_slice(
    &[1., 0., 0., 0., 0., 0.,
      0., 1., 0., 0., 0., 0.,
     ]
  );

  kf
}
