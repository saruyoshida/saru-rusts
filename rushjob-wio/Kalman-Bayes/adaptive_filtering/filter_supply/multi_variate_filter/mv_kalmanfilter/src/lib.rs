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
use filter_base::*;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct KalmanFilter<const M: usize,
                        const N: usize,
                        const C: usize>
{ // フィルタ共通変数定義
  pub fb: FilterBase<M, N, C>,  
}
#[allow(non_snake_case)]
impl <const M: usize,
      const N: usize,
      const C: usize>
     KalmanFilter<M, N, C>
{
  pub fn new() -> Self {
    Self {
      fb: FilterBase::<M, N, C>::new(),
    }
  }
  // 予測
  pub fn predict(&mut self) {
    // x = Fx + Bu
    self.fb.x = self.fb.F * self.fb.x +
                self.fb.B * self.fb.u;
    // P = FPF' + Q
    self.fb.P = (self.fb.F * self.fb.P) *
                self.fb.F.transpose() +
                self.fb.Q;
  }
  // 更新
  pub fn update(&mut self) {
    // y = z - Hx : 残差
    self.fb.y  = self.fb.z - 
                 self.fb.H * self.fb.x;
    // S = HPH' + R

    let PHt = self.fb.P * 
              self.fb.H.transpose();
    self.fb.S  = self.fb.H * PHt + self.fb.R;
    // K = PH'inv(S)
    self.fb.K  = PHt * 
                 self.fb.S.try_inverse()
                          .unwrap();
    // x = x + Ky
    self.fb.x  += self.fb.K * self.fb.y;
    // P = (I-KH)P は数値的に不安定なので
    // P = (I-KH)P(I-KH)' + KRK' を使用
    let i_kh = self.fb.I - 
               self.fb.K * self.fb.H;
    self.fb.P   = (i_kh  * self.fb.P)  * 
                  i_kh.transpose()   + 
                  (self.fb.K * self.fb.R) *
                  self.fb.K.transpose();
  }
}
// --- Clippy対応 ---
impl<const M: usize,
     const N: usize,
     const C: usize>
    Default for KalmanFilter<M, N, C> {
  fn default() -> Self {
    Self::new()
  }
}

