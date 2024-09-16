#![no_std]
// # 予測
//    シグマ点作成
//    X = sigma_fn(x, P)
//    重み
//    wm, wc = weight_fn(g, param)
//    変換後シグマ点
//    Y = fx(X, dt, u)
//     → (0..g).foreach(|i| 
//          Y.row_mut(i)
//           .copy_from(fx(X.row(i), dt, u));
//    無香料変換
//    xp, Pp = ut(Y, wm, wc, Q)
//     → mu = wm * Y.T
//     → P.full もしくは 
//        P +=
//             i in 0..g
//              (wc[i]*((Y[i]-m)*(Y[i]-m).T))
//              .sum()
//        P += Q
//
//    x: 状態の平均         m x 1
//    P: 状態の共分散行列   m x m
//    X: シグマ点           g x m
//    wm: 平均の重み        1 x g
//    wc: 分散の重み        1 X g
//    Y: 変換後シグマ点     g x m
//    Q: プロセスモデルに加わるノイズの
//       共分散行列         m x m
//    fx: 状態遷移関数     (m x 1, f32, c x 1)
//                       -> m x 1
//
//    シグマ点の作成 X = sigma_fn(x, P)
//      X.row(0) = x
//      U = ((g + λ) * P).cholesky()
//      k in 0..g
//        X.rows(k+1)   = x + U.rows(k)
//        X.rows(g+k+1) = x - U.rows(k)
//        
// # 更新
//    シグマ点を観測空間へ変換
//    Z = hx(Y)
//    → (0..2g+1).foreach(|i| 
//         Z.row_mut(i)
//          .copy_from(hx(Y.row(i))
//      
//    平均と共分散行列を無香料変換で計算
//    zp, Pz = ut(Z, wm, wc, R)
//    残差
//    y = z - zp
//    カルマンゲイン
//      状態と観測値の相互共分散 
//      → Pxz.full もしくは Pxz +=
//        i in 0..2g+1
//           (wc[i]*((Y[i]-xp)*(Z[i]-zp).T))
//           .sum()
//      カルマンゲイン
//      K = Pxz * Pz.inv
//
//    結果
//    x = xp + K * y
//    P = Pp - (K * Pz) * K.T
//
//  z: 観測値平均         n x 1
//  R: 観測値に加わるノイズの
//           共分散行列   n x n
//観測関数
//  Hx: 観測関数          n x m
//      状態 x を観測値 z に変換
//制御関数
//  u: 制御入力           c x 1
//                   1 <= c <= m
//  B: 制御入力モデル,関数m x c
//     制御入力を状態 x の変化に変換する行列
//シグマ点
//  G: シグマ点数         g = m x 2 + 1
//他
//  S: 系不確実性あるいは発展共分散行列
//                        n x n
//  y: 残差               n x 1
//  K: カルマンゲイン     m x n
// -----------------------------------------
use nalgebra::SMatrix;
use ms_sigmapoints::*;
use us_transform::*;
use filter_base::*;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct UsKalmanFilter
           <const M:  usize,
            const N:  usize,
            const C:  usize,
            const G:  usize,
            const LR: usize,
            const LC: usize>
{
  // フィルタ共通変数定義
  pub fb: FilterBase<M, N, C>,  
  // 無香料カルマンフィルタ用行列
     zp: SMatrix<f32, 1, N>, // 無香料変換後z
    Pxz: SMatrix<f32, M, N>, // 相互共分散行列
  pub lm:SMatrix<f32, LR, LC>,// ﾗﾝﾄﾞﾏｰｸ 
  // シグマポイントデータ
     Wm: SMatrix<f32, 1, G>,    // 重み:平均
     Wc: SMatrix<f32, 1, G>,    // 重み:共分散
  sigmas_f: SMatrix<f32, G, M>, // Σ点:状態
  sigmas_h: SMatrix<f32, G, N>, // Σ点:観測
  // 状態遷移関数(fx)
  #[allow(clippy::type_complexity)]
  pub fx : fn(&SMatrix<f32, M, 1>,   // x
              &SMatrix<f32, C, 1>,   // u
              &SMatrix<f32, M, M>,   // F
              &SMatrix<f32, M, C>,   // B
              f32,                   // dt
           ) -> SMatrix<f32, M, 1>, 
  // 観測関数(hx)
  #[allow(clippy::type_complexity)]
  pub hx : fn(&SMatrix<f32, M, 1>,   // x
              &SMatrix<f32, N, M>,   // H
              &SMatrix<f32, LR, LC>, // lm
           ) -> SMatrix<f32, N, 1>,
  // 引き算関数(x:状態)
  pub residual_x: fn(&SMatrix<f32, 1, M>,
                     &SMatrix<f32, 1, M>
                  ) -> SMatrix<f32, 1, M>,
  // 引き算関数(z:観測値)
  pub residual_z: fn(&SMatrix<f32, 1, N>,
                     &SMatrix<f32, 1, N>
                  ) -> SMatrix<f32, 1, N>,
  // 足し算関数(x:状態)
  pub state_add : fn(&SMatrix<f32, M, 1>,
                     &SMatrix<f32, M, 1>
                  ) -> SMatrix<f32, M, 1>,
  // シグマポイントオブジェクト
  pub sigmp: MSSigmaPoints<M, G>,
  // 無香料変換(状態変数)オブジェクト
  pub utx  : UsTransform<M, G>,
  // 無香料変換(観測変数)オブジェクト
  pub utz  : UsTransform<N, G>,
  // 移動量
  pub dt   : f32,
}
#[allow(non_snake_case)]
impl<const M:  usize,
     const N:  usize,
     const C:  usize,
     const G:  usize,
     const LR: usize,
     const LC: usize>
    UsKalmanFilter<M, N, C, G, LR, LC>
{
  pub fn new(
    sigmp: MSSigmaPoints<M, G>,
    utx  : UsTransform<M, G>,
    utz  : UsTransform<N, G>,
  ) -> Self {
    let mut ukf = Self {
     fb: FilterBase::<M, N, C>::new(),
     zp: SMatrix::<f32, 1,N> ::zeros(),
    Pxz: SMatrix::<f32, M, N> ::zeros(),
     lm: SMatrix::<f32, LR, LC> ::zeros(),
     Wc: SMatrix::<f32, 1,G> ::zeros(),
     Wm: SMatrix::<f32, 1,G> ::zeros(),
      sigmas_f: SMatrix::<f32, G, M>::zeros(),
      sigmas_h: SMatrix::<f32, G, N>::zeros(), 
      dt : 1.0, 
      fx : fx_default,
      hx : hx_default,
      residual_x: residual_x_default,
      residual_z: residual_z_default,
      state_add : state_add_default,
      sigmp,
      utx,
      utz, 
    };
    ukf.Wm.copy_from(&ukf.sigmp.Wm);
    ukf.Wc.copy_from(&ukf.sigmp.Wc);

    ukf
  }
  // 予測
  pub fn predict(&mut self) {
    // 状態遷移後シグマ点Y(sigmas_f)作成
    self.process_sigmas();
    // 遷移後シグマ点Yに対する無香料変換で
    // 事前分布の平均xと共分散行列Pを算出
    // :x, P = UT(Y, Wm, Wc, Q) →
    //   x = (0..=2*n).sum(Wm[i]*Y[i])
    //   P = (0..=2*n).sum(Wc[i]*(Y[i]-x)*
    //                         (Y[i]-x).T + Q)
    let (x, P) = self.utx.transform(
                   &self.sigmas_f, 
                   &self.Wm, 
                   &self.Wc, 
                   &self.fb.Q,
                 );
    self.fb.x.copy_from(&x.transpose());
    self.fb.P.copy_from(&P);
    // 事前分布x,Pを元にシグマ点Yを再作成
    self.sigmas_f = self.sigmp.sigma_points(
                      &self.fb.x, 
                      &self.fb.P
                    );
  }
  // 更新
  pub fn update(&mut self) {
    // 予測シグマ点Yを観測空間に変換し、
    // シグマ点Z(sigmas_h)作成
    self.observation_sigmas();
    // 観測空間シグマ点Z(sigmas_h)から
    // 無香料変換を使って観測予測値平均と
    // 観測予測値共分散行列を算出
    // :μz, Pz = UT(Z, Wm, Wc, R) →
    //   μz = (0..=2*n).sum(Wm[i]*Z[i])
    //   Pz  = (0..=2*n).sum(Wc[i]*(Z[i]-μz)*
    //                        (Z[i]-μz).T + R
    (self.zp, self.fb.S) = self.utz.transform(
                             &self.sigmas_h, 
                             &self.Wm, 
                             &self.Wc, 
                             &self.fb.R,
                           );
    // 残差計算:観測値z - 観測予測平均zp
    // y = z - μz
    self.fb.y = ((self.residual_z)(
                   &self.fb.z.transpose(),
                   &self.zp,
                )).transpose();
    // 状態と観測値の相互共分散行列(Pxz)作成
    // Pxz = (0..=2*n).sum(Wc[i]*(Y[i]-x)*
    //                        (Z[i]-μz).T
    self.cross_variance();
    // カルマンゲイン算出
    // Pxz(予測値と観測値の変動の類似性) /
    // Pz(観測値の不確実性)
    // K = Pxz * Pz.inv()
    self.fb.K = self.Pxz *
             self.fb.S.try_inverse().unwrap();
    // 状態の新しい推定値を残差と
    // カルマンゲインを使って算出
    // x = x + K * y
    self.fb.x = (self.state_add)(
                  &self.fb.x, 
                 &(self.fb.K * self.fb.y)
                );
    // 共分散行列の新しい推定値を算出
    // P = P - K * Pz * K.T
    self.fb.P -= self.fb.K * 
                 (self.fb.S *
                  self.fb.K.transpose());
  }
  // 状態遷移後シグマ点Y(sigmas_f)作成
  pub fn process_sigmas(&mut self) {
    // 重みWcとWmはコンストラクト時に生成済
    // :Wc, Wm = weight-function(n, param)
    // 事後分布の平均xと共分散行列Pから
    // シグマ点Xを作成
    // :X = sigma-function(x, P)
    let sigmas = self.sigmp.sigma_points(
                   &self.fb.x, 
                   &self.fb.P
                 );
    // ワーク行列
    let mut s = SMatrix::<f32, 1, M>
                       ::zeros();
    // シグマ点Xを状態遷移関数fxで
    // プロセスモデルによる次時間への射影
    // であるシグマ点Yに変換
    // :Y = fx(X, dt, u)
    (0..G).for_each(|i| {
      s.copy_from(&sigmas.row(i));

      self.sigmas_f.row_mut(i).copy_from(
        &((self.fx)(
            &s.transpose(), 
            &self.fb.u,
            &self.fb.F,
            &self.fb.B,
            self.dt,
          ).transpose()
         )
      );
    });
  }
  // 観測空間変換後シグマ点Z(sigmas_h)作成
  pub fn observation_sigmas(&mut self) {
    // ワーク行列
    let mut s = SMatrix::<f32, M, 1>
                       ::zeros();
    // 事前分布を表すシグマ点Yを観測関数 h(x) 
    // で観測空間に移す
    // Z = h(Y)
    (0..G).for_each(|i| {
      s.copy_from(
        &(self.sigmas_f.row(i).transpose())
      );
      self.sigmas_h.row_mut(i).copy_from(
        &((self.hx)(
            &s,
            &self.fb.H,
            &self.lm
          ).transpose()
         )
      );
    });
  }
  // 状態と観測値の相互共分散行列(Pxz)作成
  pub fn cross_variance(&mut self) {
    // 相互共分散行列初期化
    self.Pxz.fill(0.0);
    // ワーク行列
    let mut f = SMatrix::<f32, 1, M>
                       ::zeros();
    let mut h = SMatrix::<f32, 1, N>
                       ::zeros();
    // Pxz = (0..=2*n).sum(Wc[i]*(Y[i]-x)*
    //                        (Z[i]-μz).T)
    (0..G).for_each(|i| {
      f.copy_from(&(self.sigmas_f.row(i)));
      h.copy_from(&(self.sigmas_h.row(i)));
      // Y[i]-x
      let dx = (self.residual_x)(
                 &f,
                 &self.fb.x.transpose(),
               );
      // Z[i]-μz
      let dz = (self.residual_z)(
                 &h,
                 &self.zp,
               );
      // sum(Wc[i]*(Y[i]-x)*(Z[i]-μz).T)
      self.Pxz += (dx.transpose() * dz) *
                  *self.Wc.column(i)
                       .as_scalar();
    });
  }
}
// --- 関数定義型 デフォルト実装 ---
// 状態遷移関数(fx)デフォルト
#[allow(non_snake_case)]
fn fx_default<const M: usize, const C: usize>(
  x:  &SMatrix<f32, M, 1>,
  u:  &SMatrix<f32, C, 1>,
  F:  &SMatrix<f32, M, M>,
  B:  &SMatrix<f32, M, C>,
  _dt:f32,
) -> SMatrix<f32, M, 1>
{
  F * x + B * u
}
// 観測関数(hx)デフォルト
#[allow(non_snake_case)]
fn hx_default
   <const M:  usize, 
    const N:  usize,
    const LR: usize,
    const LC: usize>
(
  x:  &SMatrix<f32, M, 1>,
  H:  &SMatrix<f32, N, M>,
  _zt:&SMatrix<f32, LR, LC>,
) -> SMatrix<f32, N, 1>
{
  H * x
}
// 引き算関数(x:状態)デフォルト
fn residual_x_default<const M: usize>(
  a: &SMatrix<f32, 1, M>,
  b: &SMatrix<f32, 1, M>
) -> SMatrix<f32, 1, M>
{
  a - b
}
// 引き算関数(z:観測値)デフォルト
fn residual_z_default<const N: usize>(
  a: &SMatrix<f32, 1, N>,
  b: &SMatrix<f32, 1, N>
) -> SMatrix<f32, 1, N>
{
  a - b
}
// 足し算関数(x:状態)デフォルト
fn state_add_default<const M: usize>(
  a: &SMatrix<f32, M, 1>,
  b: &SMatrix<f32, M, 1>
) -> SMatrix<f32, M, 1>
{
  a + b
}
// -------------------------------