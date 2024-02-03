#![no_std]

extern crate nalgebra as na;

pub use na::base::dimension::*;
pub use na::{OMatrix, DimName};
pub use na::base::Matrix;
use na::DefaultAllocator;
use na::allocator::Allocator;
use micromath::F32Ext;

// MerweScaledSigmaPoints
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct MSSigmaPoints<M, G> 
where
  M: DimName,
  G: DimName,
  DefaultAllocator: 
    Allocator<f32, U1, G> +
    Allocator<f32, U1, M>
{
  pub Wm: OMatrix<f32, U1, G>,    // 重み:状態
  pub Wc: OMatrix<f32, U1, G>,    // 重み:観測
  // 引き算関数
  pub subtract: 
          fn(&OMatrix<f32, U1, M>,
             &OMatrix<f32, U1, M>
          ) -> OMatrix<f32, U1, M>,
  // シグマ点パラメータ
      alpha : f32,   // α
      beta  : f32,   // β
      kappa : f32,   // κ 
  // 他
      lambda: f32,   // λ
      n     : usize, // シグマ点分割数
}
impl<M, G> MSSigmaPoints<M, G>
where
  M: DimName,
  G: DimName, 
  DefaultAllocator: 
    Allocator<f32, G,  M>  +
    Allocator<f32, M,  M>  +
    Allocator<f32, M,  U1> +
    Allocator<f32, U1, G>  +
    Allocator<f32, U1, M>  
{
  pub fn new(
    alpha: f32,
    beta:  f32,
    kappa: f32,
  ) -> Self
  {
    // 次元数チェック
    assert!(
      (G::dim() - 1) / 2 == M::dim(),
      // G = M x 2 + 1 じゃないとダメ
      "G = M x 2 + 1, otherwise."
    );

    let mut sp = Self {
      Wc: OMatrix::<f32, U1, G>::zeros(),
      Wm: OMatrix::<f32, U1, G>::zeros(),
      subtract: subtract_default,
      alpha,
      beta,
      kappa,
      lambda: 0.0,
      n: (G::dim() - 1) / 2,
    };
    sp.compute_weights();
    sp
  }
  // 重み計算
  fn compute_weights(&mut self) {
    let n = self.n as f32;
    // λ=α**2(n+κ)−n
    let lambda = self.alpha.powi(2) * 
                 (n + self.kappa) -
                 n;
    // 1/2(n+λ)
    let c = 0.5 / (n + lambda);
    self.Wc.fill(c);
    self.Wm.fill(c);
    // Wn0 = λ/(n+λ)
    self.Wm[(0, 0)] = lambda / (n + lambda);
    // Wc0 = λ/(n+λ)+1-a**2+b
    self.Wc[(0, 0)] = lambda / (n + lambda) +
                      (1. - 
                       self.alpha.powi(2) +
                       self.beta
                      );

    self.lambda = lambda;
  }
  // シグマポイント作成
  #[allow(non_snake_case)]
  pub fn sigma_points(
    &self,
    x: &OMatrix<f32, M, U1>,
    P: &OMatrix<f32, M, M>,
  ) ->  OMatrix<f32, G, M>
  { 
    // ワーク行列
    let xt = OMatrix::<f32, U1, M>
                    ::from(x.transpose());
    let mut urk = OMatrix::<f32, U1, M>
                         ::zeros();
    // U=√(n+λ)Σ
    let n = self.n as f32;
//    println!("P:{}",((n + self.lambda) * 
//             bP).transpose());
    let U = ((n + self.lambda) * P)
            .cholesky()  // コレスキー
            .unwrap()
            .l()         // 下三角
            .transpose();// 上三角に変換
    // シグマポイント行列
    let mut sigmas = OMatrix::<f32, G, M>
                            ::zeros();
    // X0=μ
    sigmas.row_mut(0).copy_from(&xt);
    // シグマポイント設定
    (0..self.n).for_each(|k| {
        urk.copy_from(&U.row(k));
        // X(1〜n)=μ+√(n+λ)
        sigmas.row_mut(k+1).copy_from(
          &((self.subtract)(
               &xt, &(&urk * -1.)
           ))
        );
        // X(n+1〜2n)=μ-√(n+λ)
        sigmas.row_mut(self.n+k+1).copy_from(
          &((self.subtract)(
               &xt, &urk
           ))
        );
    });

    sigmas
  }
}
// --- 関数定義型 デフォルト実装 ---
// 引き算関数デフォルト
fn subtract_default<M>(
  a: &OMatrix<f32, U1, M>,
  b: &OMatrix<f32, U1, M>
) -> OMatrix<f32, U1, M>
where
  M: DimName,
  DefaultAllocator: 
    Allocator<f32, U1, M> 
{
  a - b
}

