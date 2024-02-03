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
extern crate nalgebra as na;

pub use na::base::dimension::*;
pub use na::{OMatrix, DimName};
pub use na::base::Matrix;
pub use na::DefaultAllocator;
pub use na::allocator::Allocator;

pub use ms_sigmapoints::*;
pub use us_transform::*;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct UsKalmanFilter
           <M, N, C, G, LR, LC>
where
  M:  DimName,
  N:  DimName,
  C:  DimName,
  G:  DimName,
  LR: DimName,
  LC: DimName,
  DefaultAllocator: Allocator<f32, M, U1> +
                    Allocator<f32, M, M>  +
                    Allocator<f32, M, C>  +
                    Allocator<f32, C, U1> +
                    Allocator<f32, N, M>  +
                    Allocator<f32, N, N>  +
                    Allocator<f32, N, U1> +
                    Allocator<f32, M, N>  +
                    Allocator<f32, U1,G>  +
                    Allocator<f32, G, M>  +
                    Allocator<f32, G, N>  +
                    Allocator<f32, U1, M> +
                    Allocator<f32, U1, N> + 
                    Allocator<f32, LR, LC> 
{
  // 多変量カルマンフィルタ用行列
  pub x: OMatrix<f32, M, U1>,// 状態変数
  pub P: OMatrix<f32, M, M>, // 状態共分散行列
  pub Q: OMatrix<f32, M, M>, // ﾌﾟﾛｾｽﾉｲｽﾞ行列
  pub B: OMatrix<f32, M, C>, // 制御関数
  pub u: OMatrix<f32, C, U1>,// 制御入力
  pub F: OMatrix<f32, M, M>, // 状態遷移行列
  pub H: OMatrix<f32, N, M>, // 観測行列
  pub R: OMatrix<f32, N, N>, // 観測ノイズ行列
  pub z: OMatrix<f32, N, U1>,// 観測値
  pub K: OMatrix<f32, M, N>, // カルマンゲイン
  pub y: OMatrix<f32, N, U1>,// 残差
  pub S: OMatrix<f32, N, N>, // 発展共分散行列
//    I: OMatrix<f32, M, M>, // 単位行列
  // 無香料カルマンフィルタ用行列
     zp: OMatrix<f32, U1,N>, // 無香料変換後z
    Pxz: OMatrix<f32, M, N>, // 相互共分散行列
  pub lm:OMatrix<f32, LR, LC>,  // ﾗﾝﾄﾞﾏｰｸ 
  // シグマポイントデータ
     Wm: OMatrix<f32, U1,G>,    // 重み:平均
     Wc: OMatrix<f32, U1,G>,    // 重み:共分散
  sigmas_f: OMatrix<f32, G, M>, // Σ点:状態
  sigmas_h: OMatrix<f32, G, N>, // Σ点:観測
  // 状態遷移関数(fx)
  pub fx : fn(&OMatrix<f32, M, U1>,  // x
              &OMatrix<f32, C, U1>,  // u
              &OMatrix<f32, M, M>,   // F
              &OMatrix<f32, M, C>,   // B
              f32,                   // dt
           ) -> OMatrix<f32, M, U1>, 
  // 観測関数(hx)
  pub hx : fn(&OMatrix<f32, M, U1>,  // x
              &OMatrix<f32, N, M>,   // H
              &OMatrix<f32, LR, LC>, // lm
           ) -> OMatrix<f32, N, U1>,
  // 引き算関数(x:状態)
  pub residual_x: fn(&OMatrix<f32, U1, M>,
                     &OMatrix<f32, U1, M>
                  ) -> OMatrix<f32, U1, M>,
  // 引き算関数(z:観測値)
  pub residual_z: fn(&OMatrix<f32, U1, N>,
                     &OMatrix<f32, U1, N>
                  ) -> OMatrix<f32, U1, N>,
  // 足し算関数(x:状態)
  pub state_add : fn(&OMatrix<f32, M, U1>,
                     &OMatrix<f32, M, U1>
                  ) -> OMatrix<f32, M, U1>,
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
impl<M, N, C, G, LR, LC> UsKalmanFilter
    <M, N, C, G, LR, LC>
where
  M : DimName,
  N : DimName,
  C : DimName, 
  G:  DimName,
  LR: DimName,
  LC: DimName,
  DefaultAllocator: Allocator<f32, M, U1> +
                    Allocator<f32, M, M>  +
                    Allocator<f32, M, C>  +
                    Allocator<f32, C, U1> +
                    Allocator<f32, N, M>  +
                    Allocator<f32, N, N>  +
                    Allocator<f32, N, U1> +
                    Allocator<f32, M, N>  +
                    Allocator<f32, U1,G>  +
                    Allocator<f32, G, M>  +
                    Allocator<f32, G, N>  +
                    Allocator<f32, U1, M> +
                    Allocator<f32, U1, N> +
                    Allocator<f32, LR, LC> 
{
  pub fn new(
    sigmp: MSSigmaPoints<M, G>,
    utx  : UsTransform<M, G>,
    utz  : UsTransform<N, G>,
  ) -> Self {
    let mut ukf = Self {
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
//    I: OMatrix::<f32, M, M> ::identity(),
     zp: OMatrix::<f32, U1,N> ::zeros(),
    Pxz: OMatrix::<f32, M, N> ::zeros(),
     lm: OMatrix::<f32, LR, LC> ::zeros(),
     Wc: OMatrix::<f32, U1,G> ::zeros(),
     Wm: OMatrix::<f32, U1,G> ::zeros(),
      sigmas_f: OMatrix::<f32, G, M>::zeros(),
      sigmas_h: OMatrix::<f32, G, N>::zeros(), 
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
  pub fn predict(&mut self) -> &mut Self {
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
                   &self.Q,
                 );
    self.x.copy_from(&x.transpose());
    self.P.copy_from(&P);
    // 事前分布x,Pを元にシグマ点Yを再作成
    self.sigmas_f = self.sigmp.sigma_points(
                      &self.x, 
                      &self.P
                    );

    self
  }
  // 更新
  pub fn update(&mut self) -> &mut Self {
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
    (self.zp, self.S) = self.utz.transform(
                          &self.sigmas_h, 
                          &self.Wm, 
                          &self.Wc, 
                          &self.R,
                        );
    // 残差計算:観測値z - 観測予測平均zp
    // y = z - μz
    self.y = ((self.residual_z)(
               &self.z.transpose(),
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
    self.K = &self.Pxz *
             &self.S.clone()
                    .try_inverse()
                    .unwrap();
    // 状態の新しい推定値を残差と
    // カルマンゲインを使って算出
    // x = x + K * y
    self.x = (self.state_add)(
               &self.x, 
               &(&self.K * &self.y)
             );
    // 共分散行列の新しい推定値を算出
    // P = P - K * Pz * K.T
    self.P -= &self.K * 
              (&self.S *
               &self.K.transpose());

    self
  }
  // 状態遷移後シグマ点Y(sigmas_f)作成
  pub fn process_sigmas(&mut self) {
    // 重みWcとWmはコンストラクト時に生成済
    // :Wc, Wm = weight-function(n, param)
    // 事後分布の平均xと共分散行列Pから
    // シグマ点Xを作成
    // :X = sigma-function(x, P)
    let sigmas = self.sigmp.sigma_points(
                   &self.x, 
                   &self.P
                 );
    // ワーク行列
    let mut s = OMatrix::<f32, U1, M>
                       ::zeros();
    // シグマ点Xを状態遷移関数fxで
    // プロセスモデルによる次時間への射影
    // であるシグマ点Yに変換
    // :Y = fx(X, dt, u)
    (0..G::dim()).for_each(|i| {
      s.copy_from(&sigmas.row(i));

      self.sigmas_f.row_mut(i).copy_from(
        &((self.fx)(
            &s.transpose(), 
            &self.u,
            &self.F,
            &self.B,
            self.dt,
          ).transpose()
         )
      );
    });
  }
  // 観測空間変換後シグマ点Z(sigmas_h)作成
  pub fn observation_sigmas(&mut self) {
    // ワーク行列
    let mut s = OMatrix::<f32, M, U1>
                       ::zeros();
    // 事前分布を表すシグマ点Yを観測関数 h(x) 
    // で観測空間に移す
    // Z = h(Y)
    (0..G::dim()).for_each(|i| {
      s.copy_from(
        &(self.sigmas_f.row(i).transpose())
      );
      self.sigmas_h.row_mut(i).copy_from(
        &((self.hx)(
            &s,
            &self.H,
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
    let mut f = OMatrix::<f32, U1, M>
                       ::zeros();
    let mut h = OMatrix::<f32, U1, N>
                       ::zeros();
    // Pxz = (0..=2*n).sum(Wc[i]*(Y[i]-x)*
    //                        (Z[i]-μz).T)
    (0..G::dim()).for_each(|i| {
      f.copy_from(&(self.sigmas_f.row(i)));
      h.copy_from(&(self.sigmas_h.row(i)));
      // Y[i]-x
      let dx = (self.residual_x)(
                 &f,
                 &self.x.transpose(),
               );
      // Z[i]-μz
      let dz = (self.residual_z)(
                 &h,
                 &self.zp,
               );
      // sum(Wc[i]*(Y[i]-x)*(Z[i]-μz).T)
      self.Pxz += (&dx.transpose() * &dz) *
                  *self.Wc.column(i)
                       .as_scalar();
    });
  }
}
// --- 関数定義型 デフォルト実装 ---
// 状態遷移関数(fx)デフォルト
#[allow(non_snake_case)]
fn fx_default<M, C>(
  x:  &OMatrix<f32, M, U1>,
  u:  &OMatrix<f32, C, U1>,
  F:  &OMatrix<f32, M, M>,
  B:  &OMatrix<f32, M, C>,
  _dt:f32,
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
  F * x + B * u
}
// 観測関数(hx)デフォルト
#[allow(non_snake_case)]
fn hx_default<M, N, LR, LC>(
  x:  &OMatrix<f32, M, U1>,
  H:  &OMatrix<f32, N, M>,
  _zt:&OMatrix<f32, LR, LC>,
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
  H * x
}
// 引き算関数(x:状態)デフォルト
fn residual_x_default<M>(
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
// 引き算関数(z:観測値)デフォルト
fn residual_z_default<N>(
  a: &OMatrix<f32, U1, N>,
  b: &OMatrix<f32, U1, N>
) -> OMatrix<f32, U1, N>
where
  N: DimName,
  DefaultAllocator: 
    Allocator<f32, U1, N>
{
  a - b
}
// 足し算関数(x:状態)デフォルト
fn state_add_default<M>(
  a: &OMatrix<f32, M, U1>,
  b: &OMatrix<f32, M, U1>
) -> OMatrix<f32, M, U1>
where
  M: DimName,
  DefaultAllocator: 
    Allocator<f32, M, U1>
{
  a + b
}
// -------------------------------