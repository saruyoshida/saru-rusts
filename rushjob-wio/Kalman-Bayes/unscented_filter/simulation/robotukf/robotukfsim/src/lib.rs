#![no_std]

use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use micromath::F32Ext;
use core::f32::consts::PI;
extern crate nalgebra as na;
use na::base::dimension::*;
use linspacef32::linspacef32;

// ========================================
// ----------------------------------------
// グラフ表示設定
pub const DSP_ST: (i32, i32) = (0, 5);
pub const DSP_SZ: (u32, u32) =(315, 235);
// 設定① ---- now
/*
pub const DSP_XRS:i32        = -50;
pub const DSP_XRE:i32        = 250;
pub const DSP_YRS:i32        = 0;
pub const DSP_YRE:i32        = 180;
pub const DSP_HSP:(f32, f32) = (10.0, 10.0);
pub const DSP_SN: (usize, usize) 
                             = (50, 40);
                             */
// 設定②③ ----
pub const DSP_XRS:i32        = 0;
pub const DSP_XRE:i32        = 700;
pub const DSP_YRS:i32        = 0;
pub const DSP_YRE:i32        = 350;
pub const DSP_HSP:(f32, f32) = (10.0, 10.0);
pub const DSP_SN: (usize, usize) 
                             = (100, 50);
// ----------------------------------------
// カルマンフィルタ次元設定
pub type M = U3; // 状態、プロセスモデル
pub type C = U2; // 制御入力
pub type G = U7; // シグマ点数 Mx2+1
pub type LC= U2; // ランドマーク次元数
// 設定① ----
/*
pub type LR= U3; // ランドマーク数 N/2
pub type N = U6; // 観測値
*/
// 設定② ----
/*
pub type LR= U7; // ランドマーク数 N/2
pub type N = U14; // 観測値
*/
// 設定③ ---- 
pub type LR= U2; // ランドマーク数 N/2
pub type N = U4; // 観測値
// ----------------------------------------
// シミュレーション設定
// 設定① ----
/*
pub const DT: f32 = 1.0;          // dt
pub const STEP: usize = 10;       // ﾌｨﾙﾀ
pub const ELLIPSE_STEP: usize = 1;// 描画
*/
// 設定②③ ----
pub const DT: f32 = 0.1;          // dt
pub const STEP: usize = 1;        // ﾌｨﾙﾀ
pub const ELLIPSE_STEP: usize = 20;// 描画
// ----------------------------------------
// ローカルシミュレーション設定
// 設定① ----
/*
const LMC: usize = 3;   // ﾗﾝﾄﾞﾏｰｸ数
*/
// 設定② ---- 
/*
const LMC: usize = 7;   // ﾗﾝﾄﾞﾏｰｸ数
*/
// 設定③ ---- now
const LMC: usize = 2;   // ﾗﾝﾄﾞﾏｰｸ数
// -----------
// ホイルベース
const WHEELBASE: f32 = 0.5;
// ----------------------------------------
// ランドマーク作成
fn make_landmarks() -> [[f32; 2]; LMC] {
// 設定① ----
/*
  [[5., 10.], [10., 5.], [15., 15.]]
*/
// 設定② ----
/*
  [[5. , 10.], [10.,  5.], [15., 15.], 
   [20.,  5.], [0. , 30.], [50., 30.],
   [40., 10.]]
   */
// 設定③ ---- now
  [[5., 10.], [10., 5.]]
}
// 制御入力作成
pub fn make_cmd()
  -> impl Iterator<Item=[f32; 2]>
{
// 設定① ----
/*
  [[1.1, 0.01]].into_iter().cycle().take(200)
*/
// 設定②③ ---- now
  // 静止状態からの加速
  linspacef32(0.001, 1.1, 30).map(
    |c| [c, 0.]
  ).chain(
    [[1.1, 0.]]
    .into_iter().cycle().take(50)
  ).chain(
  // 左への旋回
    turn(1.1, 0., 2., 15)
  ).chain(
    [[1.1, 2.0_f32.to_radians()]]
    .into_iter().cycle().take(100)
  ).chain(
  // 右への旋回
    turn(1.1, 2., -2., 15)
  ).chain(
    [[1.1, -2.0_f32.to_radians()]]
    .into_iter().cycle().take(200)
  ).chain(
    turn(1.1, -2., 0., 15)
  ).chain(
    [[1.1, 0.0]]
    .into_iter().cycle().take(150)
  ).chain(
    turn(1.1, 0., 1., 25)
  ).chain(
    [[1.1, 1.0_f32.to_radians()]]
    .into_iter().cycle().take(100)
  )
}
// ========================================
fn turn(
  v : f32,
  t0: f32,
  t1: f32,
  steps: usize
) -> impl Iterator<Item=[f32; 2]>
{
  linspacef32(
    t0.to_radians(),
    t1.to_radians(),
    steps
  ).map(move |r| [v, r])
}

pub struct RobotUkfSim {
  pub landmarks  : [[f32; 2]; LMC], // ﾗﾝﾄﾞﾏｰｸ
  pub sim_pos    : [f32; 3],
  pub z          : [f32; LMC*2],
  pub sigma_range: f32,
  pub sigma_bearing: f32,
  rng            : StdRng,
  normal         : Normal<f32>,
}
// new
impl RobotUkfSim {
  pub fn new() -> Self {
    // 乱数設定
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();
    // ランドマーク
    let landmarks = make_landmarks();

    RobotUkfSim {
      landmarks,
      sim_pos: [2., 6., 0.3], // 位置、角度
      z: [0.0; LMC*2],        // 観測値
      sigma_range: 0.3,       // 
      sigma_bearing: 0.1,     // 
      rng,
      normal,
    }
  }
}
impl RobotUkfSim {
  pub fn move_next(
    &mut self,
    dt : f32,
    u  : &[f32; 2]
  )
  {
    self.sim_pos = RobotUkfSim::move_to(
                     &self.sim_pos,
                     dt,
                     u
                   );
  }
  // 観測値
  pub fn z(&mut self) -> &[f32; LMC*2] {
    let mut randn = ||
      self.normal.sample(&mut self.rng);

    for (i, lm) in self.landmarks
                       .into_iter() 
                       .enumerate()
    {
      let (x, y) = (self.sim_pos[0],
                    self.sim_pos[1]);

      let (dx, dy) = (lm[0] - x, lm[1] - y);

      let d = (dx.powi(2) + dy.powi(2)).sqrt() 
            + randn() * self.sigma_range;

      let bearing = dy.atan2(dx);

      let a = RobotUkfSim::normalize_angle(
                bearing    - 
                self.sim_pos[2] +
                randn() * self.sigma_bearing
              );

      self.z[i*2]   = d;
      self.z[i*2+1] = a;
    }
    &(self.z)
  }
// セッター
  // 乱数シード
  pub fn set_random_seed(
    &mut self, 
    random_seed: u8
  ) -> &mut Self {
    self.rng = StdRng::from_seed(
                [random_seed; 32]
               );
    self
  }
// ========================================
// 関連関数
  // 運動モデル  
  pub fn move_to(  
    x:  &[f32],
    dt: f32,
    u:  &[f32],
  ) -> [f32; 3]
  {
    let hdg = x[2]; // 現在向き
    let vel = u[0]; // 制御入力:速度
                    // 制御入力:角度
    let steering_angle = u[1];
    let dist = vel * dt;
    let mut xc = [0.0_f32; 3];
    xc.copy_from_slice(x);

    // ロボットが旋回している場合
    if steering_angle.abs() > 0.001 {
       // 回転半径Rの角度β
       let beta = (dist / WHEELBASE) * 
                  steering_angle.tan();
       // 回転半径R
       let r = WHEELBASE / 
               steering_angle.tan();
       // −Rsin(θ)+Rsin(θ+β)
       [-r*hdg.sin() + r*(hdg + beta).sin(),
         r*hdg.cos() - r*(hdg + beta).cos(), 
         beta
       ].iter().enumerate()
       .for_each(|(i, v)| xc[i] += v);
    } else { 
    // 直線に沿って移動している場合
       [dist * hdg.cos(), 
        dist * hdg.sin(), 
        0.
       ].iter().enumerate()
       .for_each(|(i, v)| xc[i] += v);
    }
    xc
  } 
  // 角度の正規化
  pub fn normalize_angle(x: f32) -> f32 {
    // x を [0, 2 pi) の範囲に変換する。
    let mut a = x % (2. * PI);
    if a < 0. { a += 2. * PI; }
    // x を [-pi, pi) に移す。
    if a > PI { a -= 2. * PI; }
    a
  }
}
// ========================================

