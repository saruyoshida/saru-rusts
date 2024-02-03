#![no_std]
extern crate nalgebra as na;
use na::base::dimension::*;
use na::{OMatrix, DimName};
use na::DefaultAllocator;
use na::allocator::Allocator;

use micromath::F32Ext;
use robotukfsim::RobotUkfSim;
// ========================================
// 無香料カルマンフィルタ関数定義
pub struct RobotUkfFn;

impl RobotUkfFn {
  // 状態遷移関数(fx)
  pub fn fx<M, C>(
    x:  &OMatrix<f32, M, U1>,
    u:  &OMatrix<f32, C, U1>,
    _:  &OMatrix<f32, M, M>,
    _:  &OMatrix<f32, M, C>,
    dt: f32,
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
    OMatrix::<f32, M, U1>::from_iterator(
      RobotUkfSim::move_to(
        x.as_slice(), dt, u.as_slice()
      ).into_iter()
    )
  }
  // 観測関数
  pub fn hx<M, N, LR, LC>(
    x:  &OMatrix<f32, M, U1>,
    _: &OMatrix<f32, N, M>,
    landmarks: &OMatrix<f32, LR, LC>,
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
    let mut h = OMatrix::<f32, N, U1>
                       ::zeros();
    landmarks.row_iter().enumerate()
    .for_each(|(i, m)| {
      let ih = i * LC::dim();
      h.row_mut(ih).copy_from_slice(
        &[
          // ランドマークとの直距離
          ((m[0] - x[(0, 0)]).powi(2) +
           (m[1] - x[(1, 0)]).powi(2)
          ).sqrt()
         ]
      );
      h.row_mut(ih+1).copy_from_slice(
        &[
          // ランドマークとの相対角度
          RobotUkfSim::normalize_angle(
            // 位置との仰角
            (m[1] - x[(1, 0)]).atan2
            (m[0] - x[(0, 0)]) -
            // ロボットの角度を引く
            x[(2, 0)]  
          ) // 角度の正規化
         ]
      );
    });
    h
  }
  // 引き算関数(x:状態)
  pub fn residual_x<M>(
    a: &OMatrix<f32, U1, M>,
    b: &OMatrix<f32, U1, M>
  ) -> OMatrix<f32, U1, M>
  where
    M: DimName,
    DefaultAllocator: 
      Allocator<f32, U1, M>
  {
    let mut y = a - b;
    y[(0, 2)] = 
      RobotUkfSim::normalize_angle(y[(0, 2)]);
    y
  }
  // 引き算関数(z:観測値)
  pub fn residual_h<N>(
    a: &OMatrix<f32, U1, N>,
    b: &OMatrix<f32, U1, N>
  ) -> OMatrix<f32, U1, N>
  where
    N: DimName,
    DefaultAllocator: 
      Allocator<f32, U1, N>
  {
    let mut y = a - b;
    (0..N::dim()).into_iter().step_by(2)
    .for_each(|i|
      y[(0,i+1)] =          
        RobotUkfSim::normalize_angle(
                       y[(0,i+1)]
                     )
    );
    y
  }
  // --- 無香料変換(状態)用 関数定義 ---
  // 状態平均計算関数
  pub fn state_mean<MN, G>(
    sigmas: &OMatrix<f32, G,  MN>,
    wm    : &OMatrix<f32, U1, G >
  ) -> OMatrix<f32, U1, MN>
  where
    MN: DimName,
    G : DimName,
    DefaultAllocator: 
      Allocator<f32, G,  MN> +
      Allocator<f32, U1, G > +
      Allocator<f32, U1, MN> +
      Allocator<f32, G , U1> +
      Allocator<f32, G , G>  +
      Allocator<f32, G >
  { 
    // Σsinθi
    let sum_sin = (
      wm * sigmas.column(2).map(|v| v.sin())
    ).sum(); 
    // Σcosθi
    let sum_cos = (
      wm * sigmas.column(2).map(|v| v.cos())
    ).sum();
    // 平均格納 ロボットの[x, y, 向き(角度)]
    OMatrix::<f32, U1, MN>::from_iterator(
      [(wm * sigmas.column(0)).sum(),
       (wm * sigmas.column(1)).sum(),
       sum_sin.atan2(sum_cos),
      ].into_iter()
    )
  }
  // --- 無香料変換(観測)用 関数定義 ---
  // 観測平均計算関数
  pub fn z_mean<MN, G>(
    sigmas: &OMatrix<f32, G,  MN>,
    wm    : &OMatrix<f32, U1, G >
  ) -> OMatrix<f32, U1, MN>
  where
    MN: DimName,
    G : DimName,
    DefaultAllocator: 
      Allocator<f32, G,  MN> +
      Allocator<f32, U1, G > +
      Allocator<f32, U1, MN> +
      Allocator<f32, G , G>  +
      Allocator<f32, G >
  {
    // 平均格納行列
    let mut x =  OMatrix::<f32, U1, MN>
                        ::zeros();
    // センサー1個分(2列)ずつ区切って計算
    (0..MN::dim()).into_iter().step_by(2)
    .for_each(|i| {
      // Σsinθi 
      let sum_sin = (
        wm * 
        sigmas.column(i+1).map(|v| v.sin())
      ).sum();
      // Σcosθi
      let sum_cos = (
        wm *
        sigmas.column(i+1).map(|v| v.cos())
      ).sum();
      // 平均格納
      // センサーとの[直距離, 角度, ...]
      x.column_mut(i).copy_from_slice(
        &[(wm * sigmas.column(i)).sum()]
      );
      x.column_mut(i+1).copy_from_slice(
        &[sum_sin.atan2(sum_cos)]
      );
    });
    x
  }
}
