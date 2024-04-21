#![no_std]

use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use micromath::F32Ext;
use particlefilter::*;
use heapless::Vec;

// ----------------------------------------
// グラフ表示設定
// 枠設定
pub const DSP_ST: (i32, i32) = (0, 5);
pub const DSP_SZ: (u32, u32) =(315, 235);
// 目盛設定
pub const DSP_XRS:i32        = 0;
pub const DSP_XRE:i32        = 100;
pub const DSP_YRS:i32        = 0;
pub const DSP_YRE:i32        = 100;
pub const DSP_HSP:(f32, f32) = (10.0, 10.0);
pub const DSP_SN: (usize, usize) 
                             = (20, 20);
// 描画ステップ設定
const D_STEP: usize = 1;
// ----------------------------------------
// 粒子フィルタ次元設定
const PM : usize = 3;       // 状態変数次元
const PC : usize = 2;       // 制御入力次元
const PG : usize = 3000 ;   // 粒子数
const PD : usize = 2;       // 位置次元
const PN : usize = 1;       // 観測値次元
const LMC: usize = 10;      // ﾗﾝﾄﾞﾏｰｸ数(最大)
// 粒子フィルタパラメータ設定
const SEED: u8   = 23;      // 乱数シード
                            // センサーノイズ
const SENSER_STD_ERR:[f32; PN] = [0.8];
                            // 状態遷移ノイズ
const CMD_ERR :[f32; PC] = [2.0, 2.0];
                            // 初期位置
const INIT_POS:[f32; PD] = [0.5, 0.5];
const DT : f32   = 1.0;     // dt
const CT : usize = 20;      // 繰返し数
                            // 粒子生成ﾊﾟﾗﾒｰﾀ
const CP_PARAM : [[f32; 2]; PM] =
      [[-1.,10.], [-1.,10.], [0., 6.28]];
//    [[1., 5.], [1., 5.], [PI/4., PI/4.]];
                            // ﾘｻﾝﾌﾟﾘﾝｸﾞ判定数
const JVAL: f32 = PG as f32 / 2.;
// ----------------------------------------
// 制御入力作成
fn make_cmd()
  -> impl Iterator<Item=[f32; PC]>
{
  [[0.5, 0.5]].into_iter().cycle().take(CT)
}
// ----------------------------------------
pub struct SimConfig {
  pub seed : u8,            // 乱数シード
  pub pos  : [f32; PD],     // 実際位置
  pub param: [[f32; 2]; PM],// 粒子生成ﾊﾟﾗﾒｰﾀ
  pub jval : f32,           // ﾘｻﾝﾌﾟﾘﾝｸﾞ判定数
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
    pf.create_fn = uniform_particles
                   ::<PM, PD>;
    // リサンプリング関数
/*
    pf.resample_fn = multinomal_resample
                     ::<PM>;
*/
     // 残差再サンプリング
    pf.resample_fn = residual_resample
                     ::<PM>;
    // 状態遷移関数
    pf.fx = fx;
    //
    pf
  }
  // 位置の移動
  pub fn move_next(&mut self) {
    // 正規分布乱数
    let normal = Normal::new(0., 1.).unwrap();
    let mut randn = || normal.sample(
                         &mut self.rng
                       );
    // 位置の移動
    self.pos[0] += self.u[0];
    self.pos[1] += self.u[0];
    // ﾗﾝﾄﾞﾏｰｸの取得
     // ここではﾗﾝﾄﾞﾏｰｸ=位置としておく
    self.lmv.clear();
    self.lmv.push(self.pos).unwrap();
    // 観測値の作成
     // ここでは直距離=ﾉｲｽﾞ分としておく
    self.zv.clear();
    self.zv.push(
      [randn() * SENSER_STD_ERR[0]]
    ).unwrap();
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
  pt : &mut [[f32; PM]],     // 粒子(pt)
  u  : &[f32],               // 制御入力
  q  : &[f32],               // 制御ﾉｲｽﾞ
  dt : f32,                  // dt
  mut rng: &mut StdRng,      // 乱数
)
{
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
}