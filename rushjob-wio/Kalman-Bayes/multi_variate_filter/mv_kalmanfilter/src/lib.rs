#![no_std]
// # 予測
//    x = F @ x + B @ u
//    P = F @ P @ F.T + Q
//
// # 更新
//    S = H @ P @ H.T + R
//    K = P @ H.T @ inv(S)
//    y = z - H @ x
//    x += K @ y
//    P = P - K @ H @ P
//      P=(I−KH)_P → P=(1−cK)P →
//      P = (I-KH)P(I-KH)' + KRK'
//
//状態
//  x: 状態の平均         m x 1
//  P: 状態の共分散行列   m x m
//プロセスモデル
//  F: 状態遷移関数、行列 m x m
//  Q: プロセスモデルに加わるノイズの
//     共分散行列         m x m
//観測値
//  z: 観測値平均         n x 1
//  R: 観測値に加わるノイズの
//           共分散行列   n x n
//観測関数
//  H: 観測関数           n x m
//     状態 x を観測値 z に変換
//制御関数
//  u: 制御入力           c x 1
//                   1 <= c <= m
//  B: 制御入力モデル,関数m x c
//     制御入力を状態 x の変化に変換する行列
//他
//  S: 系不確実性あるいは発展共分散行列
//                        n x n
//  y: 残差               n x 1
//  K: カルマンゲイン     m x n
// -----------------------------------------
// 最初の観測値(z)からフィルタを初期化する例
//  x: x = H.pinv @ z
//     zを観測関数Hで状態xに変換
//     x = H.pseudo_inverse().unwrap() * z;
//  P: [[R[0], 0], [0, vel.max**2]]
//     R[0]: 位置観測値のノイズ分散
//     vel.max: 速度の最大値
//     P.set_partial_diagonal(
//       [R[(0, 0)], vel.pow(2)].into_iter()
//     );
// -----------------------------------------
extern crate nalgebra as na;

pub use na::base::dimension::*;
pub use na::{OMatrix, DimName};
pub use na::base::Matrix;
use na::DefaultAllocator;
use na::allocator::Allocator;

#[derive(Clone)]
#[allow(non_snake_case)]
pub struct KalmanFilter<M, N, C>
where
  M: DimName,
  N: DimName,
  C: DimName,
  DefaultAllocator: Allocator<f32, M, U1> +
                    Allocator<f32, M, M>  +
                    Allocator<f32, M, C>  +
                    Allocator<f32, C, U1> +
                    Allocator<f32, N, M>  +
                    Allocator<f32, N, N>  +
                    Allocator<f32, N, U1> +
                    Allocator<f32, M, N>,
{
  pub x: OMatrix<f32, M, U1>,// 状態変数
  pub P: OMatrix<f32, M, M>, // 状態共分散行列
  pub Q: OMatrix<f32, M, M>, // ﾌﾟﾛｾｽﾉｲｽﾞ行列
  pub B: OMatrix<f32, M, C>, // 制御関数
  pub u: OMatrix<f32, C, U1>,// 制御入力
  pub F: OMatrix<f32, M, M>, // 状態遷移関数
  pub H: OMatrix<f32, N, M>, // 観測関数
  pub R: OMatrix<f32, N, N>, // 観測ノイズ行列
  pub z: OMatrix<f32, N, U1>,// 観測値
  pub K: OMatrix<f32, M, N>, // カルマンゲイン
  pub y: OMatrix<f32, N, U1>,// 残差
  pub S: OMatrix<f32, N, N>, // 発展共分散行列
      I: OMatrix<f32, M, M>, // 単位行列
}

impl<M, N, C> KalmanFilter<M, N, C>
where
  M: DimName,
  N: DimName,
  C: DimName,
  DefaultAllocator: Allocator<f32, M, U1> +
                    Allocator<f32, M, M>  +
                    Allocator<f32, M, C>  +
                    Allocator<f32, C, U1> +
                    Allocator<f32, N, M>  +
                    Allocator<f32, N, N>  +
                    Allocator<f32, N, U1> +
                    Allocator<f32, M, N>,
{
  pub fn new() -> Self {
    Self {
      x: OMatrix::<f32, M, U1>::zeros(),
      P: OMatrix::<f32, M, M> ::identity(),
      Q: OMatrix::<f32, M, M> ::identity(),
      B: OMatrix::<f32, M, C> ::zeros(),
      u: OMatrix::<f32, C, U1>::zeros(),
      F: OMatrix::<f32, M, M> ::identity(),
      H: OMatrix::<f32, N, M> ::zeros(),
      R: OMatrix::<f32, N, N> ::identity(),
      z: OMatrix::<f32, N, U1>::zeros(),
      K: OMatrix::<f32, M, N> ::zeros(),
      y: OMatrix::<f32, N, U1>::zeros(),
      S: OMatrix::<f32, N, N> ::zeros(),
      I: OMatrix::<f32, M, M> ::identity(),
    }
  }
  // 予測
  pub fn predict(&mut self) -> &mut Self {
    // x = Fx + Bu
    self.x = &self.F * &self.x +
             &self.B * &self.u;
    // P = FPF' + Q
    self.P = (&self.F * &self.P) *
             &self.F.transpose() +
             &self.Q;
    self
  }
  // 更新
  pub fn update(&mut self) -> &mut Self {
    // y = z - Hx : 残差
    self.y  = &self.z - &self.H * &self.x;
    // S = HPH' + R
    #[allow(non_snake_case)]
    let PHt = &self.P * &self.H.transpose();
    self.S  = &self.H * &PHt + &self.R;
    // K = PH'inv(S)
    self.K  = &PHt * 
              &self.S.clone()
                     .try_inverse()
                     .unwrap();
    // x = x + Ky
    self.x   = &self.x + &self.K * &self.y;
    // P = (I-KH)P は数値的に不安定なので
    // P = (I-KH)P(I-KH)' + KRK' を使用
    let i_kh = &self.I - &self.K * &self.H;
    self.P   = (&i_kh  * &self.P)  * 
               &i_kh.transpose()   + 
               (&self.K * &self.R) *
               &self.K.transpose();
    self
  }
}