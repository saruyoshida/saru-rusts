#![no_std]

extern crate nalgebra as na;

use na::base::dimension::*;
use na::{OMatrix, DimName};
use na::DefaultAllocator;
use na::allocator::Allocator;

// 無香料変換
#[derive(Clone, Debug)]
pub struct UsTransform<MN, G>
where
  MN: DimName,
  G : DimName,
  DefaultAllocator: 
    Allocator<f32, G,  MN> +
    Allocator<f32, U1, G > +
    Allocator<f32, U1, MN> 
{
  // 平均計算関数
  pub mean_fn: fn(&OMatrix<f32, G,  MN>,
                  &OMatrix<f32, U1, G >
               ) -> OMatrix<f32, U1, MN>,
  // 引き算関数
  pub residual_fn:
               fn(&OMatrix<f32, U1, MN>,
                  &OMatrix<f32, U1, MN>
               ) -> OMatrix<f32, U1, MN>,
}
#[allow(non_snake_case)]
impl<MN, G> UsTransform<MN, G>
where
  MN: DimName,
  G : DimName,
  DefaultAllocator: 
    Allocator<f32, G,  MN>  +
    Allocator<f32, U1, G >  +
    Allocator<f32, MN, MN>  +
    Allocator<f32, U1, MN>  +
    Allocator<f32, MN, U1>  +
    Allocator<f32, U1, U1> 
{ 
  pub fn new() -> Self {
    Self {
      mean_fn    : mean_fn_default,
      residual_fn: residual_fn_default,
    }
  }
  // 変換
  pub fn transform(
    &self,
    sigmas    : &OMatrix<f32, G,  MN>,
    Wm        : &OMatrix<f32, U1, G >,
    Wc        : &OMatrix<f32, U1, G >, 
    noise_cov : &OMatrix<f32, MN, MN>, 
  ) -> (OMatrix<f32, U1, MN>,
        OMatrix<f32, MN, MN>)
  {
    // x:平均 μ=Σi{Wmi*Xi}
    let x = (self.mean_fn)(&sigmas, &Wm);
    // P:分散(初期化)
    let mut P  = OMatrix::<f32, MN, MN>
                        ::zeros();
    // ワーク行列
    let mut Xi = OMatrix::<f32, U1, MN>
                        ::zeros();
    // P:分散(設定)
    (0..G::dim()).for_each(|i| {
      // Xi
      Xi.copy_from(&sigmas.row(i));
      // Xi-μ
      let y = (self.residual_fn)(&Xi, &x);
      // Σi{Wci(Xi-μ)(Xi-μ).T}
      P += *Wc.column(i).as_scalar() * 
           (&y.transpose() * &y);
    });
    // P=Σi{Wci(Xi-μ)(Xi-μ).T} + Q
    P += noise_cov;
    (x, P)
  }
}
// --- 関数定義型 デフォルト実装 ---
// 平均計算関数デフォルト
fn mean_fn_default<MN, G>(
  a: &OMatrix<f32, G,  MN>,
  b: &OMatrix<f32, U1, G >
) -> OMatrix<f32, U1, MN>
where
  MN: DimName,
  G : DimName,
  DefaultAllocator: 
    Allocator<f32, G,  MN> +
    Allocator<f32, U1, G > +
    Allocator<f32, U1, MN>
{
  b * a  
}
// 引き算関数デフォルト
fn residual_fn_default<MN>(
  a: &OMatrix<f32, U1, MN>,
  b: &OMatrix<f32, U1, MN>
) -> OMatrix<f32, U1, MN>
where
  MN: DimName,
  DefaultAllocator: 
    Allocator<f32, U1, MN> 
{
  a - b
}
