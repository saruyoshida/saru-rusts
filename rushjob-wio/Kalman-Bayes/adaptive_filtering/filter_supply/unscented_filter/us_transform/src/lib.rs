#![no_std]
use nalgebra::SMatrix;

// 無香料変換
#[derive(Clone, Debug)]
pub struct UsTransform
           <const MN: usize, const G: usize>
{
  // 平均計算関数
  pub mean_fn: fn(&SMatrix<f32, G, MN>,
                  &SMatrix<f32, 1, G >
               ) -> SMatrix<f32, 1, MN>,
  // 引き算関数
  pub residual_fn:
               fn(&SMatrix<f32, 1, MN>,
                  &SMatrix<f32, 1, MN>
               ) -> SMatrix<f32, 1, MN>,
}
#[allow(non_snake_case)]
impl<const MN: usize, const G: usize>
     UsTransform<MN, G> 
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
    sigmas    : &SMatrix<f32, G,  MN>,
    Wm        : &SMatrix<f32, 1, G >,
    Wc        : &SMatrix<f32, 1, G >, 
    noise_cov : &SMatrix<f32, MN, MN>, 
  ) -> (SMatrix<f32, 1, MN>,
        SMatrix<f32, MN, MN>)
  {
    // x:平均 μ=Σi{Wmi*Xi}
    let x = (self.mean_fn)(sigmas, Wm);
    // P:分散(初期化)
    let mut P  = SMatrix::<f32, MN, MN>
                        ::zeros();
    // ワーク行列
    let mut Xi = SMatrix::<f32, 1, MN>
                        ::zeros();
    // P:分散(設定)
    (0..G).for_each(|i| {
      // Xi
      Xi.copy_from(&sigmas.row(i));
      // Xi-μ
      let y = (self.residual_fn)(&Xi, &x);
      // Σi{Wci(Xi-μ)(Xi-μ).T}
      P += *Wc.column(i).as_scalar() * 
           (y.transpose() * y);
    });
    // P=Σi{Wci(Xi-μ)(Xi-μ).T} + Q
    P += noise_cov;
    (x, P)
  }
}
// --- 関数定義型 デフォルト実装 ---
// 平均計算関数デフォルト
fn mean_fn_default
   <const MN: usize, const G: usize>(
  a: &SMatrix<f32, G,  MN>,
  b: &SMatrix<f32, 1, G >
) -> SMatrix<f32, 1, MN>
{
  b * a  
}
// 引き算関数デフォルト
fn residual_fn_default<const MN: usize>(
  a: &SMatrix<f32, 1, MN>,
  b: &SMatrix<f32, 1, MN>
) -> SMatrix<f32, 1, MN>
{
  a - b
}
// --- Clippy指摘対応 ---
impl<const MN: usize, const G: usize> 
    Default for UsTransform<MN, G> {
  fn default() -> Self {
    Self::new()
  }
}
