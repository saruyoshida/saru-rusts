#![no_std]
use nalgebra::SMatrix;
use imm_estimator::*;
// フィルタ供給:M,N,CとFLCはこの中で定義
pub use filter_supply_imm_practise::*;

// モード確率:モード数=フィルタ数
fn mode_probability() 
                  -> SMatrix<f32, FLC, 1>  {
  SMatrix::<f32, FLC, 1>::from_column_slice(
    &[0.5, 0.5]
  )
}
// 遷移確率行列
fn transition_probability()
                  -> SMatrix<f32, FLC, FLC>  {
  SMatrix::<f32, FLC, FLC>::from_column_slice(
    &[0.97, 0.03,
      0.03, 0.97,
     ]
  )
}
// IMM供給
pub struct IMMSupply {
  pub filters: [Filters; FLC],
  pub imm    : IMMEstimator<M, FLC>,
  pub z      : SMatrix<f32, N, 1>, // 観測値
  pub u      : SMatrix<f32, C, 1>, // 制御入力
}
#[allow(non_snake_case)]
impl IMMSupply {
  pub fn new() -> Self {
    let mut imm = Self {
      filters: filter_supply(),
      imm    : IMMEstimator::<M, FLC>
               ::new(
                 mode_probability(),
                 transition_probability(),
               ),
      z      : SMatrix::<f32, N, 1>::zeros(),
      u      : SMatrix::<f32, C, 1>::zeros(),
    };
    // 混合確率計算
    imm.imm.compute_mixing_probabilities();
    // IMMEstimatorにfiltersのxとPを渡す
    imm.xP_copy_to_estimator();
    // 混合推定値算出
    imm.imm.compute_state_estimate();

    imm
  }
  // 予測
  pub fn predict(&mut self) {
    // IMMEstimatorにfiltersのxとPを渡す
    self.xP_copy_to_estimator();
    // IMMEstimatorの予測処理を行う
    self.imm.predict();
    // IMMEstimatorで算出のxとPをfiltersに戻す
    self.xP_copy_from_estimator();
    // 各フィルタで予測処理を実行
    self.filters.iter_mut().for_each(|f| 
      // 制御入力設定:今回未使用(例)
      // self.u.iter().enumrate() 
      //    .for_each|(i, u)| f.u_set(i, u) 
      // )  

      // 各フィルタでの予測
      f.predict()
    );
    // IMMEstimatorにfiltersのxとPを渡す
    self.xP_copy_to_estimator();
    // 混合推定値算出
    self.imm.compute_state_estimate();
  }
  // 更新
  pub fn update(&mut self) {
    for (i, f) in self.filters.iter_mut()
                              .enumerate() {
                        // 観測値設定
      self.z.iter().enumerate() 
          .for_each(|(i, z)| f.z_set(i, *z)
      ); 
      f.update();       // 更新
                        // 尤度を設定
      self.imm.f_lh[(i, 0)] = f.likelihood();
    }
    // IMMEstimatorにfiltersのxとPを渡す
    self.xP_copy_to_estimator();
    // IMMEstimatorを更新
    self.imm.update();
  }
  // IMMEstimatorにfiltersのxとPを渡す
  fn xP_copy_to_estimator(&mut self) {
    self.filters.iter()
                .zip(self.imm.f_x.iter_mut())
                .zip(self.imm.f_P.iter_mut())
                .for_each(|((f, x), P)| {
      x.copy_from_slice(f.x_as_slice());
      P.copy_from_slice(f.P_as_slice());
    });
  }
  // IMMEstimatorで算出のxとPをfiltersに戻す
  fn xP_copy_from_estimator(&mut self) {
      self.filters.iter_mut()
                .zip(self.imm.f_x.iter())
                .zip(self.imm.f_P.iter())
                .for_each(|((f, x), P)| {
      f.x_from_slice(x.as_slice());
      f.P_from_slice(P.as_slice());
    });
  }
}
//--- Clippy対応---
impl Default for IMMSupply {
      fn default() -> Self {
        Self::new()
    }
}
