#![no_std]
use nalgebra::SMatrix;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct IMMEstimator<const M  : usize,
                        const FLC: usize>
{ // FLC=フィルタ数=モード数
  // 混合状態変数
  pub x    : SMatrix<f32, M,   1>,
  // 混合共分散行列
  pub P    : SMatrix<f32, M,   M>,
  // モード確率
  pub mu   : SMatrix<f32, FLC, 1>,
  // マルコフ連鎖に対する遷移確率行列
  pub tpM  : SMatrix<f32, FLC, FLC>,
  // 正規化係数:全確率
  pub cbar : SMatrix<f32, FLC, 1>,
  // 混合確率
  pub omega: SMatrix<f32, FLC, FLC>,
  // 尤度(filters.likelihood)
  pub f_lh : SMatrix<f32, FLC, 1>,
  // 状態変数(filters.x)
  pub f_x  :[SMatrix<f32, M,   1>; FLC],
  // 共分散行列(filters.P)
  pub f_P  :[SMatrix<f32, M,   M>; FLC],
}
#[allow(non_snake_case)]
impl <const M  : usize,
      const FLC: usize>
     IMMEstimator<M, FLC>
{
  pub fn new(
    mu  : SMatrix<f32, FLC, 1>,
    tpM : SMatrix<f32, FLC, FLC>,
  ) -> Self {
    Self {
      x    : SMatrix::<f32, M,   1>::zeros(),
      P    : SMatrix::<f32, M,   M>::zeros(),
      mu   : mu / mu.sum(), // ﾓｰﾄﾞ確率正規化
      tpM,
      cbar : SMatrix::<f32, FLC, 1>::zeros(),
      omega: SMatrix::<f32, FLC, FLC>
                                   ::zeros(),
      f_lh : SMatrix::<f32, FLC, 1>::zeros(),
      f_x  :[SMatrix::<f32, M,   1>::zeros()
             ;FLC],
      f_P  :[SMatrix::<f32, M,   M>::zeros()
             ;FLC],
    }
  }
  // 予測
  pub fn predict(&mut self) {
    // この前に呼出元で各フィルタのxとPを
    // f_xとf_Pにコピーしておく
    let mut xms = self.f_x;
    xms.iter_mut().for_each(|xm| xm.fill(0.));
    
    let mut Pms = self.f_P;
    Pms.iter_mut().for_each(|Pm| Pm.fill(0.));

    for ((ws, xm), Pm) in 
                   self.omega.column_iter()
                       .zip(xms.iter_mut())
                       .zip(Pms.iter_mut()) {
      // xm[j]=Σ(i=1 to N)w[i][j]*x[i]
      for (x, w) in self.f_x.iter()
                    .zip(ws.iter()) 
      {
        *xm += *w * *x;
      }
      // Pm[j]=Σ(i=1 to N)
      //  w[i][j] * 
      //  (x[i]-xm[i]}*(x[i]-xm[i]}.T + P[i])
      for ((x, w), P) in self.f_x.iter()
                         .zip(ws.iter())
                         .zip(self.f_P.iter())
      {
        let y = *x - *xm;
        *Pm += *w * (y * y.transpose() + *P);
      }
    }
    self.f_x = xms;
    self.f_P = Pms;
    // この後に呼出元で各フィルタのxとPを
    // f_xとf_Pの値に置き換えて後
    // 各フィルタでpredictを実行
    // 各フィルタのxとPをf_zとf_Pにコピーし
    // compute_state_estimateを呼出す
  }
  // 更新
  pub fn update(&mut self) {
    // この前に呼出元で各フィルタのxとPと
    // likelihoodを
    // f_xとf_Pとf_lhにコピーしておく

    // モード確率を更新:確率*尤度
    // mu[i]=∥ L[i]⋅cbar[i] ∥ 
    self.mu = self.f_lh
                  .component_mul(&self.cbar);
    self.mu /= self.mu.sum(); // 正規化
    // 混合確率算出
    self.compute_mixing_probabilities();
    // IMMアルゴリズム推定値算出
    self.compute_state_estimate();
  }
  // 混合確率算出
  pub fn compute_mixing_probabilities(
    &mut self
  ) 
  {
  // ω[i][j]=∥ μ[i]⋅M[i][j] ∥
  // μ[i](モード確率)を事前分布、
  // M[i][j](マルコフ連鎖)を尤度とみなして
  // 混合確率:ω を(μ[i]xM[i][j])
  //              /cbar(正規化係数:全確率)
  // で算出

    // cbar(正規化係数:全確率)算出
    // muxM
    self.cbar = self.tpM * self.mu;
    // 
    for i in 0..FLC {for j in 0..FLC {
      self.omega[(i, j)] = 
          (self.tpM[(i, j)]*self.mu[(i, 0)]) / 
          self.cbar[(j, 0)];
    }}
  }
  // 混合推定値算出
  pub fn compute_state_estimate(&mut self) {
    // x=Σ(j=1 to N)mu[j]*x[j]
    self.x.fill(0.);
    for (x, mu) in self.f_x.iter()
                   .zip(self.mu.iter()) {
      self.x += *x * *mu;
    }
    // P=Σ(j=1 to N)mu[j]*
    //   ((x[j]-x)*(x[j]-x).T + P[j])
    self.P.fill(0.);
    for ((x, P), mu) in self.f_x.iter()
                        .zip(self.f_P.iter()) 
                        .zip(self.mu.iter()) {
      let y = *x - self.x;
      self.P += *mu * 
                (y * y.transpose() + *P);
    }
  }
}
  