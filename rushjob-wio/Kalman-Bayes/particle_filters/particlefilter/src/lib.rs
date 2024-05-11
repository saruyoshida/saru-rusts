#![no_std]

use core::f32::consts::PI;
use rand_distr::{Distribution, Uniform, 
                 Normal};
use rand::prelude::*;
use micromath::F32Ext;

// 粒子フィルタ
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct ParticleFilter
  <const PM : usize,         // 粒子次元
   const PC : usize,         // 制御入力次元
   const PG : usize,         // 粒子数
   const PD : usize,         // 位置次元
   const PN : usize,         // 観測値次元
  >
  // 粒子次元(PM)の0〜PD-1を位置の次元とし
  // PD〜PD+PD-2を方向の次元とする。
  // それ以降の次元は使えるかわからんが
  // 速度や加速度などの次元とする。
  // PM=PDとして全て位置次元としてもよい。
  // その場合、観測値次元PNは1とする。
{
  pub pt  : [[f32; PM]; PG], // 粒子
  pub pts : [[f32; PM]; PG], // 粒子(ｻﾝﾌﾟﾘﾝｸﾞ)
  pub wg  : [f32; PG],       // 重み
  pub R   : [f32; PN],       // 観測ﾉｲｽﾞ
  pub u   : [f32; PC],       // 制御入力
  pub Q   : [f32; PC],       // ﾌﾟﾛｾｽﾉｲｽﾞ
  pub dt  : f32,             // Dt
  pub rng : StdRng,          // 乱数
  pub create_fn:             // 粒子生成関数
    fn(&mut [[f32; PM]],      // 粒子(pt)
       &[[f32; 2]],           // ﾊﾟﾗﾒｰﾀ
       &mut StdRng,           // 乱数
      ),
  pub resample_fn:           // 再ｻﾝﾌﾟﾘﾝｸﾞ関数
    fn(&mut [f32],            // 重み(wg)
       &[[f32; PM]],          // 粒子(pt)
       &mut [[f32; PM]],      // 粒子(pts)
       &mut StdRng,           // 乱数
      ),
  pub fx:                    // 状態遷移関数
    fn(&mut [[f32; PM]],      // 粒子(pt)
       &[f32],                // 制御入力(u)
       &[f32],                // ﾌﾟﾛｾｽﾉｲｽ(Q)
       f32,                   // dt
       &mut StdRng,           // 乱数
      ),
  pub hx:                    // 観測関数
    fn([f32; PM],             // 粒子(pt)
       [f32; PD],             // ﾗﾝﾄﾞﾏｰｸ
       usize,                 // 位置次元数
      ) -> [f32; PN],         // 観測形式値
}
impl
  <const PM : usize,         // 粒子次元
   const PC : usize,         // 制御入力次元
   const PG : usize,         // 粒子数
   const PD : usize,         // 位置次元
   const PN : usize,         // 観測値次元
  >
  ParticleFilter<PM, PC, PG, PD, PN>
{
  pub fn new() -> Self {
    // 次元数チェック
    assert!(PM >= PD,
            // PM >= PD じゃないとダメ
            "PM >= PD, otherwise.");

    Self {
      pt  : [[0.0; PM]; PG], // 粒子
      pts : [[0.0; PM]; PG], // 粒子(ｻﾝﾌﾟﾘﾝｸﾞ)                       
      wg  : [0.0; PG],       // 重み
      R   : [0.0; PN],       // 観測ﾉｲｽﾞ
      u   : [0.0; PC],       // 制御入力
      Q   : [0.0; PC],       // ﾌﾟﾛｾｽﾉｲｽﾞ
      dt  : 1.0,             // Dt
                             // 乱数
      rng :StdRng::from_seed([2; 32]),
      // 粒子生成関数ﾃﾞﾌｫﾙﾄ:一様分布
      create_fn:   uniform_particles
                   ::<PM, PD>,
      // 再ｻﾝﾌﾟﾘﾝｸﾞ関数ﾃﾞﾌｫﾙﾄ:層化再ｻﾝﾌﾟﾘﾝｸﾞ
      resample_fn: stratified_resample
                   ::<PM>,
      // 状態遷移関数ﾃﾞﾌｫﾙﾄ
      fx:          fx_default
                   ::<PM, PD>,
      // 観測関数ﾃﾞﾌｫﾙﾄ
      hx:          hx_default
                   ::<PM, PD, PN>,
    }
  }
  // 粒子生成
  pub fn create_particles(
    &mut self,
    param: &[[f32; 2]],
  ) -> &mut Self
  {
    (self.create_fn)(
      &mut self.pt,
      param,
      &mut self.rng,
    );
    // 重みの初期化
    self.wg.iter_mut().for_each(|w|
      *w = 1.0 / PG as f32
    );
    //
    self
  }
  // 予測
  pub fn predict(&mut self) -> &mut Self {
    // 制御入力、ﾉｲｽﾞをもとに粒子を移動
    (self.fx)(
       &mut self.pt,
       &self.u,
       &self.Q,
       self.dt,
       &mut self.rng,
    );
    self
  }
  // 更新
  pub fn update(
    &mut self,
    // ﾗﾝﾄﾞﾏｰｸ毎の[[位置]], [観測値]
    lm: &[[f32; PD]],
    z : &[[f32; PN]],
  ) -> &mut Self 
  {
    let mut wgsum = 0.0;
    for (r, pt) in self.pt.iter()
                          .enumerate() {
      for i in 0..z.len() {
        // 粒子を観測値形式に変換
        let h = (self.hx)(*pt, lm[i], PD);
        for c in 0..h.len() {
          // 粒子と観測値の比を算出
          let pdf  = pdf(
                       h[c], 
                       self.R[c],
                       z[i][c]
                     );
          // 比を重みに掛ける
          self.wg[r] *= pdf;
      }}
      self.wg[r] += 1.0E-32f32;
      wgsum += self.wg[r];
    }
    // 重みの正規化
    self.wg.iter_mut().for_each(|w|
      *w /= wgsum
    );
    //
    self
  }
  // 実効サンプルサイズ
  pub fn neff(&self) -> f32 {
    1.0 /
    self.wg.iter()
        .fold(0.0, |s, wg| s + wg.powi(2))
  }
  // 再サンプリング
  pub fn resample(&mut self) -> &mut Self {
    // 再ｻﾝﾌﾟﾘﾝｸﾞ関数
    (self.resample_fn)(
       &mut self.wg,
       &self.pt, 
       &mut self.pts,
       &mut self.rng,
    );
    // 粒子入替え
    core::mem::swap(
      &mut self.pt,
      &mut self.pts,
    );
    // 重みの初期化
    self.wg.iter_mut().for_each(|w|
      *w = 1.0 / PG as f32
    );
    //
    self
  }
  // 平均、分散
  pub fn estimate(&self)
    -> ([f32; PD] ,[f32; PD]) 
  {
    let mut mean = [0.0; PD];
    let mut var  = [0.0; PD];
    // 平均
    for r in 0..self.pt.len() {
      for c in 0..mean.len() {
        mean[c] += self.pt[r][c] * self.wg[r];
    }}
    // 分散
    for r in 0..self.pt.len() {
      for c in 0..mean.len() {
        var[c] += (self.pt[r][c] - mean[c])
                  .powi(2) * self.wg[r];
    }}
    
    (mean, var)
  }
  // 制御入力設定
  pub fn set_u(
    &mut self,
    u: [f32; PC],
  ) -> &mut Self {
    self. u = u;
    self
  }
  // 乱数シードセッター
  pub fn set_random_seed(
    &mut self, 
    random_seed: u8
  ) -> &mut Self {
    self.rng = StdRng::from_seed(
                 [random_seed; 32]
               );
    self
  }
}
// =========================================
// === 関数定義型:粒子生成 ===
 // 粒子生成: 一様分布
  pub fn uniform_particles
  <const PM : usize,         // 粒子次元
   const PD : usize,         // 位置次元
  >
  (
    pt : &mut [[f32; PM]],
    // 次元毎の範囲指定: [[0.0,20.0],[...],..]
    uni_range: &[[f32; 2]],
    mut rng: &mut StdRng,
  ) 
  {
    uni_range.iter().enumerate()
             .for_each(|(c, a)| 
    {
      // 次元毎に指定の範囲で一様分布生成
      let uni = Uniform::from(a[0]..a[1]);
      // 粒子の列毎に一様分布を設定
      (0..pt.len()).for_each(|r| {
        pt[r][c] = uni.sample(&mut rng);
        // 方向の要素は0〜2πに調整
        if c >= PD && c < PD+PD-1 {
          pt[r][c] = ad_angl(pt[r][c]);
        }
      });
    });
  }
  // 粒子生成: ガウス分布
  pub fn gaussian_particles
  <const PM : usize,         // 粒子次元
   const PD : usize,         // 位置次元
  >
  (
    pt : &mut [[f32; PM]],
    // 次元毎の[平均,ノイズ(標準偏差)]
    mean_std: &[[f32; 2]],
    mut rng: &mut StdRng,
  )
  {
    let normal = Normal::new(0.0, 1.0)
                         .unwrap();
    let mut randn = ||normal.sample(&mut rng);

    mean_std.iter().enumerate()
            .for_each(|(c, a)| 
    {
      // 粒子の列毎にガウス分布を設定
      (0..pt.len()).for_each(|r| {
        pt[r][c] = a[0] + randn() * a[1];
        // 方向の要素は0〜2πに調整
        if c >= PD && c < PD+PD-1 {
          pt[r][c] = ad_angl(pt[r][c]);
        }
      });
    });
  }
// =========================================
// === 関数定義型:状態遷移関数ﾃﾞﾌｫﾙﾄ実装 ===
  pub fn fx_default
  <const PM : usize,         // 粒子次元
   const PD : usize,         // 位置次元
  >
  (
    pt  : &mut [[f32; PM]],  // 粒子(pt)
    _u  : &[f32],            // 制御入力
    q   : &[f32],            // ﾌﾟﾛｾｽﾉｲｽﾞ
    _dt : f32,               // dt
    mut rng: &mut StdRng,    // 乱数
  )
  {
    // ﾃﾞﾌｫﾙﾄ実装ではﾌﾟﾛｾｽﾉｲｽﾞQ[0]が
    // 入ってる前提で粒子にノイズを加える。
    let normal = Normal::new(0.0, 1.0)
                         .unwrap();
    let mut randn = ||normal.sample(&mut rng);
    
    for r in 0..pt.len() {
      for c in (0..PM).rev() {
        pt[r][c] += randn() * q[0];
        // 方向の要素は0〜2πに調整
        if c >= PD && c < PD+PD-1 {
          pt[r][c] = ad_angl(pt[r][c]);
        }
    }}
// 制御入力/ﾉｲｽﾞがある場合は、使用する側で
// 関数記述し、当関数定義に設定する。
/* 使用する側の例---------------------
  // 正規分布乱数
  let normal = Normal::new(0.0, 1.0).unwrap();
  let mut randn = || normal.sample(&mut rng);
  // 粒子の移動
  for i in 0..pt.len() { 
    // 向きの更新
    pt[i][2] += u[0] + randn() * q[0];
    pt[i][2] = ad_angl(pt[i][2]);
    // 位置の更新
    let dist = u[1] * dt + randn() * q[1];
    pt[i][0] += pt[i][2].cos() * dist;
    pt[i][1] += pt[i][2].sin() * dist;                   
  } 
  // ---------------------------------
*/
  }
// === 関数定義型:観測関数ﾃﾞﾌｫﾙﾄ実装 ===
  pub fn hx_default
  <const PM : usize,         // 粒子次元
   const PD : usize,         // 位置次元
   const PN : usize,         // 観測値次元
  >
  (
    pt: [f32; PM],           // 粒子(pt)
    lm: [f32; PD],           // ﾗﾝﾄﾞﾏｰｸ
    d : usize,               // 位置次元数
  ) -> [f32; PN]             // 観測形式値
  {
    let mut hx = [0.0f32; PN];
    // 粒子とﾗﾝﾄﾞﾏｰｸの直距離を算出
    hx[0] = multi_norm(
              pt.into_iter().take(d),
              lm.into_iter(),
              d as i32,
            );
    hx
  }
// =========================================
// === 関数定義型:再ｻﾝﾌﾟﾘﾝｸﾞ ===
  // 多項再サンプリング
  pub fn multinomal_resample
  <const PM : usize>         // 粒子次元
  (
    wg : &mut [f32],
    pt : &[[f32; PM]],
    pts: &mut [[f32; PM]],
    rng: &mut StdRng,
  )
  {
    // cumsum
    (1..wg.len()).for_each(|r| 
      wg[r] = wg[r] + wg[r-1]
    );
    wg[wg.len()-1] = 1.0;

    (0..wg.len()).for_each(|r| 
      pts[r] = pt[wg.partition_point(|&x| 
                   x < rng.gen::<f32>()
                 )]
    );
  }
  // 残差再サンプリング
  pub fn residual_resample
  <const PM : usize>         // 粒子次元
  (
    wg : &mut [f32],
    pt : &[[f32; PM]],
    pts: &mut [[f32; PM]],
    rng: &mut StdRng,
  )
  {
    let g = wg.len() as f32;
    let mut k = 0;
    // 粒子rをint(g*w)[r]個だけ複製する。
    for r in 0..wg.len() {
      for _ in 0..(wg[r] * g) as usize {
        pts[k] = pt[r];
        k += 1;
    }}
    // 残りは小数点部だけを対象に
    // 多項再サンプリング
    let mut sum = 0.0;
    (0..wg.len()).for_each(|r| {
      wg[r] = (wg[r] * g).fract();
      sum += wg[r];
    });
    // 正規化とcumsumをいっぺんに行う
    wg[0] = wg[0] / sum;
    (1..wg.len()).for_each(|r| 
      wg[r] = wg[r] / sum + wg[r-1]
    );
    wg[wg.len()-1] = 1.0;

    (k..wg.len()).for_each(|r| 
      pts[r] = pt[wg.partition_point(|&x| 
                   x < rng.gen::<f32>()
                 )]
    );
  }
  // 層化再サンプリング
  pub fn stratified_resample
  <const PM : usize>         // 粒子次元
  (
    wg : &mut [f32],
    pt : &[[f32; PM]],
    pts: &mut [[f32; PM]],
    rng: &mut StdRng,
  )
  {
    let g  = wg.len() as f32;
    let pg = g as usize;
    let pos = (0..pg).map(|i|
      (i, (i as f32 + rng.gen::<f32>()) / g)
    );
    // cumsum
    (1..wg.len()).for_each(|r| 
      wg[r] = wg[r] + wg[r-1]
    );

    let mut j = 0;
    pos.for_each(|(i, p)| {
      while !(p < wg[j]) && j < pg-1 {j += 1}
      pts[i] = pt[j];
    });
  }
  // 等間隔再サンプリング
  pub fn systematic_resample
  <const PM : usize>         // 粒子次元
  (
    wg : &mut [f32],
    pt : &[[f32; PM]],
    pts: &mut [[f32; PM]],
    rng: &mut StdRng,
  )
  {
    let g  = wg.len() as f32;
    let pg = g as usize;
    let d = rng.gen::<f32>();
    let pos = (0..pg).map(|i|
      (i, (i as f32 + d) / g)
    );
    // cumsum
    (1..wg.len()).for_each(|r| 
      wg[r] = wg[r] + wg[r-1]
    );

    let mut j = 0;
    pos.for_each(|(i, p)| {
      while !(p < wg[j]) && j < pg-1 {j += 1}
      pts[i] = pt[j];
    });
  }
// =========================================
// === 他関数 ===
  // 向き調整:0〜2πに収める
  pub fn ad_angl(mut a: f32) -> f32 {
    a %= 2.0 * PI;
    if a < 0.0 {a += 2.0 * PI;}
    a
  }
  // 直距離算出
  // 20240512 間違ってたので修正
  pub fn multi_norm(
    a: impl Iterator<Item=f32>,
    b: impl Iterator<Item=f32>,
    _d: i32, // 使わねえ
  ) -> f32 
  {
   /* 使わねえ
    let df = 1.0 / d as f32;   // 冪乗根用
   */
    a.zip(b)
    /* 修正
     .map(|(a, b)| (a - b).abs().powi(d))
    */
     .map(|(a, b)| (a - b).powi(2))
     .fold(0.0, |x, y| x + y)
    /* 修正
     .powf(df)
    */
     .sqrt()
  }
  // 確率密度関数:正規分布
  pub fn pdf(
    mean: f32,
    std : f32,
    x   : f32,
  ) -> f32
  {
    // 1/√2πσ*exp(-(x-μ)^2/2σ^2)
    1.0 / ((2.0 * PI).sqrt() * std) 
    *
    ((x - mean).powi(2) / 
     (-2.0 * std.powi(2))
    ).exp()
  }


