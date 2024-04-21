#![no_std]

use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use particlefilter::*;
use heapless::Vec;

//use core::f32::consts::PI;
// ----------------------------------------
// グラフ表示設定
// 枠設定
pub const DSP_ST: (i32, i32) = (0, 5);
pub const DSP_SZ: (u32, u32) =(315, 235);
// 目盛設定
pub const DSP_XRS:i32        = 0;
pub const DSP_XRE:i32        = 220;
pub const DSP_YRS:i32        = -40;
pub const DSP_YRE:i32        = 70;
pub const DSP_HSP:(f32, f32) = (1.0, 10.0);
pub const DSP_SN: (usize, usize) 
                             = (20, 20);
// 描画ステップ設定
const D_STEP: usize = 1;
// ----------------------------------------
// 粒子フィルタ次元設定
const PM : usize = 2;       // 状態変数次元
const PC : usize = 1;       // 制御入力次元
const PG : usize = 50;      // 粒子数
const PD : usize = 2;       // 位置次元
const PN : usize = 1;       // 観測値次元
const LMC: usize = 10;      // ﾗﾝﾄﾞﾏｰｸ数(最大)
// 粒子フィルタパラメータ設定
const SEED: u8   = 71;      // 乱数シード
const SIGMA: f32 = 1.0;
const ALPHA: f32 = 0.2;
                            // センサーノイズ
const SENSER_STD_ERR:[f32; PN] = [SIGMA];
                            // 状態遷移ノイズ
const CMD_ERR :[f32; PC] = [ALPHA * SIGMA];
                            // 初期位置
const INIT_POS:[f32; PD] = [0., DT];
const DT : f32   = 1.0;     // dt
const CT : usize = 200;     // 繰返し数
                            // 粒子生成ﾊﾟﾗﾒｰﾀ
const CP_PARAM : [[f32; 2]; PM] =
//    [[-1.,19.], [-1.,19.], [0., 6.28]];
      [[0., 3.], [0., 3.]];
                            // ﾘｻﾝﾌﾟﾘﾝｸﾞ判定数
const JVAL: f32 = PG as f32 / 2.;
// ----------------------------------------
// 制御入力作成
fn make_cmd()
  -> impl Iterator<Item=[f32; PC]>
{
  (1..=CT).into_iter().map(|u| [u as f32])
}
// ----------------------------------------
pub struct SimConfig {
  pub seed : u8,            // 乱数シード
  pub pos  : [f32; PD],     // 実際位置
  pub param: [[f32; 2]; PM],// 粒子生成ﾊﾟﾗﾒｰﾀ
  pub jval : f32,           // ﾘｻﾝﾌﾟﾘﾝｸﾞ判定
  pub u    : [f32; PC],     // 制御入力
  pub dstep: usize,         // 描画ステップ
                            // ﾗﾝﾄﾞﾏｰｸ
  pub lmv  : Vec<[f32; PD], LMC>,
  pub zv   : Vec<[f32; PN], LMC>, // 観測値
  pub zp   : [f32; PD],     // 観測位置平均
  pub rng  : StdRng,        // 乱数
}
// new
impl SimConfig {
  pub fn new() -> Self {
    SimConfig {
      seed : SEED,
      pos  : INIT_POS,
      param: CP_PARAM,
      jval : JVAL,
      u    : [0.0; PC],
      dstep: D_STEP,
      lmv  : Vec::<[f32; PD], LMC>::new(),
      zv   : Vec::<[f32; PN], LMC>::new(),
      zp   : [0.0; PD],
      rng  : StdRng::from_seed([SEED; 32]),
    }
  }
  // 粒子フィルタ生成
  pub fn particlefilter(&self) 
    -> ParticleFilter<PM, PC, PG, PD, PN>
  {
    let mut pf = ParticleFilter::
                 <PM, PC, PG, PD, PN>::new();
    pf.set_random_seed(self.seed);
    pf.R  = SENSER_STD_ERR;
    pf.Q  = CMD_ERR;
    pf.dt = DT;
    // 粒子生成関数
/*
     // 一様分布
    pf.create_fn = uniform_particles
                   ::<PM, PD>;
*/
     // ガウス分布
    pf.create_fn = gaussian_particles
                   ::<PM, PD>;
/*
    // リサンプリング関数
     // 多項再サンプリング
    pf.resample_fn = multinomal_resample
                     ::<PM>;
     // 残差再サンプリング
    pf.resample_fn = residual_resample
                     ::<PM>;
     // 層化再サンプリング
    pf.resample_fn = stratified_resample
                     ::<PM>;
     // 等間隔再サンプリング
    pf.resample_fn = systematic_resample
                     ::<PM>;
*/
    // 状態遷移関数
    pf.fx = fx;
    // 観測関数
    pf.hx = hx;
    //
    pf
  }
  // 位置の移動
  pub fn move_next(&mut self) {
    // 位置の移動
    self.pos[0] += DT;
    let normal = Normal::new(
                   0.0, CMD_ERR[0]
                 ).unwrap();
    self.pos[1] += normal.sample(
                            &mut self.rng
                           );
    // ﾗﾝﾄﾞﾏｰｸ設定
    self.lmv.clear();
    self.lmv.push(
      [self.pos[0], 0.0]
    ).unwrap();
    // 観測値の作成
    self.zv.clear();
    let normal = Normal::new(
                   0.0, SENSER_STD_ERR[0]
                 ).unwrap();
    self.zv.push(
      [self.pos[1] + normal.sample(
                            &mut self.rng
                           )
      ]
    ).unwrap();
    // 観測平均値の作成
    self.zp[0] = self.pos[0];
    self.zp[1] = self.zv[0][0];
  }
  // ランドマーク供給
  pub fn lms(&self) -> &[[f32; PD]] {
    self.lmv.as_slice()
  }
  // 観測値供給
  pub fn zs(&self) -> &[[f32; PN]] {
    self.zv.as_slice()
  }
  // 制御入力供給
  pub fn make_cmd(&self)
    -> impl Iterator<Item=[f32; PC]> {
    make_cmd()
  }
  // 制御入力設定
  pub fn set_u(
    &mut self,
    u: [f32; PC],
  ) -> &mut Self {
    self. u = u;
    self
  }
}
// ========================================
// === 関数定義型:状態遷移関数 ===
  pub fn fx(
    pt  : &mut [[f32; PM]],  // 粒子(pt)
    u   : &[f32],            // 制御入力
    q   : &[f32],            // 状態遷移ﾉｲｽﾞ
    _dt : f32,               // dt
    mut rng: &mut StdRng,    // 乱数
  )
  {
    for r in 0..pt.len() {
      pt[r][0]  = u[0];
      let normal = Normal::new(
                     pt[r][1], q[0]
                   ).unwrap();
      pt[r][1] = normal.sample(&mut rng);
    }
  }
// === 関数定義型:観測関数 ===
  pub fn hx(
    pt: [f32; PM],          // 粒子(pt)
    _ : [f32; PD],          // ﾗﾝﾄﾞﾏｰｸ
    _ : usize,              // 位置次元数
  ) -> [f32; PN]            // 観測形式値
  {
    // 粒子のy値を返却
    [pt[1]]
  }


