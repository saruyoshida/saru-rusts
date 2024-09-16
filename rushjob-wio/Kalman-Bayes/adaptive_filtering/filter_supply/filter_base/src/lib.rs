#![no_std]
use nalgebra::{SMatrix, DimMin, Const};
use core::f32::consts::PI;
#[allow(unused_imports)]
use micromath::F32Ext;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct FilterBase<const M: usize,
                      const N: usize,
                      const C: usize>
{
  pub x: SMatrix<f32, M, 1>,// 状態変数
  pub P: SMatrix<f32, M, M>,// 状態共分散行列
  pub Q: SMatrix<f32, M, M>,// ﾌﾟﾛｾｽﾉｲｽﾞ行列
  pub B: SMatrix<f32, M, C>,// 制御関数
  pub u: SMatrix<f32, C, 1>,// 制御入力
  pub F: SMatrix<f32, M, M>,// 状態遷移関数
  pub H: SMatrix<f32, N, M>,// 観測関数
  pub R: SMatrix<f32, N, N>,// 観測ノイズ行列
  pub z: SMatrix<f32, N, 1>,// 観測値
  pub K: SMatrix<f32, M, N>,// カルマンゲイン
  pub y: SMatrix<f32, N, 1>,// 残差
  pub S: SMatrix<f32, N, N>,// 発展共分散行列
  pub I: SMatrix<f32, M, M>,// 単位行列
  pub alpha   : Option<f32>,// 減衰記憶ﾊﾟﾗﾒｰﾀ
  pub cum_lh  : Option<f32>,// 累積尤度
  pub ns_count: Option<usize>,// ﾉｲｽﾞ調整ｶｳﾝﾄ
  pub phi     : Option<f32>,// ﾉｲｽﾞ分散値
  pub noise_adjust_fn:      // ﾌﾟﾛｾｽﾉｲｽﾞ調整
      fn (&mut SMatrix<f32, M, M>, // Q
              &SMatrix<f32, N, 1>, // y
              &SMatrix<f32, N, N>, // S
          &mut f32,                // eps,etc
          &mut usize,              // ns_count
          f32,  // Q_scale_factor
          f32,  // Q_adjust_param
         ),
  pub Q_scale_factor: f32, // ﾉｲｽﾞ調整係数
  pub Q_adjust_param: f32, // ﾉｲｽﾞ調整閾値
}
#[allow(non_snake_case)]
impl <const M: usize,
      const N: usize,
      const C: usize>
     FilterBase<M, N, C>
{
  pub fn new() -> Self {
    Self {
      x: SMatrix::<f32, M, 1> ::zeros(),
      P: SMatrix::<f32, M, M> ::identity(),
      Q: SMatrix::<f32, M, M> ::identity(),
      B: SMatrix::<f32, M, C> ::zeros(),
      u: SMatrix::<f32, C, 1> ::zeros(),
      F: SMatrix::<f32, M, M> ::identity(),
      H: SMatrix::<f32, N, M> ::zeros(),
      R: SMatrix::<f32, N, N> ::identity(),
      z: SMatrix::<f32, N, 1> ::zeros(),
      K: SMatrix::<f32, M, N> ::zeros(),
      y: SMatrix::<f32, N, 1> ::zeros(),
      S: SMatrix::<f32, N, N> ::zeros(),
      I: SMatrix::<f32, M, M> ::identity(),
      alpha   : None,
      cum_lh  : None,
      ns_count: None,
      phi     : None,
      noise_adjust_fn:
                   noise_adjust_fn_default,
      Q_scale_factor: 0.0,
      Q_adjust_param: 0.0,
    }
  }
  // 減衰記憶計算
  pub fn attenuation(&mut self) {
    if let Some(a) = self.alpha {
      self.P = (self.P - self.Q) * a.powi(2)
               + self.Q;
    }
  }
  // 累積尤度値取得
  pub fn cum_lh(&self) -> f32 {
    if let Some(c) = self.cum_lh {c} 
                            else {0.0}
  }
  // ﾌﾟﾛｾｽﾉｲｽﾞ調整
  pub fn noise_adjust(&mut self) {
    if let Some(mut n) = self.ns_count {
      let mut val = 
        if let Some(p) = self.phi {
                           p
                         } else {
                           self.y_eps()
                         };
      (self.noise_adjust_fn)(
         &mut self.Q,
         &self.y,
         &self.S,
         &mut val,
         &mut n,
         self.Q_scale_factor,
         self.Q_adjust_param,
      );
      self.ns_count = Some(n);
      if self.phi.is_some() {
        self.phi = Some(val);
      }
    }
  }
  // 正規化された残差 
  pub fn y_eps(&self) -> f32 {
    // y.T*S^-1*y
    *(self.y.transpose() * 
      self.S.try_inverse().unwrap() * 
      self.y
    ).as_scalar()
  }
}
// 関数定義型:ﾌﾟﾛｾｽﾉｲｽﾞ調整ﾃﾞﾌｫﾙﾄ実装(例)
#[allow(non_snake_case)]
pub fn noise_adjust_fn_default
       <const M: usize, const N: usize>
(
  Q: &mut SMatrix<f32, M, M>, // Q
  _:     &SMatrix<f32, N, 1>, // y
  _:     &SMatrix<f32, N, N>, // S
  eps     : &mut f32,         // eps
  ns_count: &mut usize,       // ns_count
  Q_scale_factor: f32,
  eps_max : f32,
)
{
  if *eps > eps_max {
    *Q *= Q_scale_factor;
    *ns_count += 1;
  } else if *ns_count > 0 {
    *Q /= Q_scale_factor;
    *ns_count -= 1;
  }
}
// determinantを使用のためimpl分離が必要
impl<const M: usize,
     const N: usize,
     const C: usize> 
    FilterBase<M, N, C> 
where
    Const<N>: DimMin<Const<N>, 
                     Output = Const<N>>,
{
  // 尤度計算
  pub fn likelihood(&self) -> f32 {
    // 1/√(2π^n * |S|) * exp(-1/2*y.T*S^-1*y)
    1. / ((2. * PI).powi(N as i32) *
          self.S.determinant()
         ).sqrt() *
    (-0.5 * self.y_eps()).exp()
  }
  // 対数尤度計算
  pub fn log_likelihood(&self) -> f32 {
    self.likelihood().ln()
  }
  // 累積尤度計算
  pub fn cum_likelihood(&mut self) {
    if let Some(c) = self.cum_lh {
      self.cum_lh = Some(c *
                         self.likelihood()
                    );
    }
  }
}

// フィルタ供給enum用implマクロ ==============
#[macro_export]
macro_rules! filter_supply_impl 
{
 ($($kf:ident,)*) => {
// --メソッド--
  // 予測
  pub fn predict(&mut self) {
    match self {
      $(Self::$kf(f) => {
          f.predict();
          f.fb.attenuation();
      },)*
    }
  }
  // 更新
  pub fn update(&mut self) {
    match self {
      $(Self::$kf(f) => {
          f.update();
          f.fb.cum_likelihood();
          f.fb.noise_adjust();
      },)*
    }
  }
//-- 値設定 --
  // 状態変数
  pub fn x_set(&mut self, r: usize, s: f32) {
    match self {
      $(Self::$kf(f) => f.fb.x[(r, 0)] = s,)*
    }
  }
  pub fn x_from_slice(&mut self, s: &[f32])
  {
    match self {
      $(Self::$kf(f) => 
                 f.fb.x.copy_from_slice(s),)*
    }
  }
  // 状態共分散行列
  pub fn P_set(&mut self, 
               r: usize, c: usize, s: f32) 
  {
    match self {
      $(Self::$kf(f) => f.fb.P[(r, c)] = s,)*
    }
  }
  pub fn P_from_slice(&mut self, s: &[f32])
  {
    match self {
      $(Self::$kf(f) => 
                 f.fb.P.copy_from_slice(s),)*
    }
  }
  // 制御入力
  pub fn u_set(&mut self, r: usize, s: f32) {
    match self {
      $(Self::$kf(f) => f.fb.u[(r, 0)] = s,)*
    }
  }
  // 観測値
  pub fn z_set(&mut self, r: usize, s: f32) {
    match self {
      $(Self::$kf(f) => f.fb.z[(r, 0)] = s,)*
    }
  }
  // 累積尤度値
  pub fn cum_lh_set(&mut self, lh: f32) {
    match self {
      $(Self::$kf(f) => 
         f.fb.cum_lh = Some(lh),)*
    }
  }
//-- 値取得 --
  // 状態変数
  pub fn x(&self, r: usize) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.x[(r, 0)],)*
    }
  }
  // 状態共分散行列
  pub fn P(&self, r: usize, c: usize) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.P[(r, c)],)*
    }
  }
  // 残差
  pub fn y(&self, r: usize) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.y[(r, 0)],)*
    }
  }
  // 観測ノイズ行列
  pub fn R(&self, r: usize, c: usize) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.R[(r, c)],)*
    }
  }
  // 発展共分散行列
  pub fn S(&self, r: usize, c: usize) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.S[(r, c)],)*
    }
  }
  // 正規化された残差 y.T*S^-1*y
  pub fn y_eps(&self) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.y_eps(),)*
    }
  }
  // 尤度
  pub fn likelihood(&self) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.likelihood(),)*
    }
  }
  // 対数尤度
  pub fn log_likelihood(&self) -> f32 {
    match self {
      $(Self::$kf(f) => 
                 f.fb.log_likelihood(),)*
    }
  }
  // 累積尤度値
  pub fn cum_lh(&self) -> f32 {
    match self {
      $(Self::$kf(f) => f.fb.cum_lh(),)*
    }
  }
  //-- スライス取得 --
  // 状態変数
  pub fn x_as_slice(&self) -> &[f32] {
    match self {
      $(Self::$kf(f) => f.fb.x.as_slice(),)*
    }
  }
  // 状態共分散行列
  pub fn P_as_slice(&self) -> &[f32] {
    match self {
      $(Self::$kf(f) => f.fb.P.as_slice(),)*
    }
  }
 }
}
// ===========================================
// --- Clippy対応 ---
impl<const M: usize,
     const N: usize,
     const C: usize> 
    Default for FilterBase<M, N, C> {
  fn default() -> Self {
    Self::new()
  }
}

