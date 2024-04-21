#![no_std]
use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use particlefilter::*;
use heapless::Vec;
use core::f32::consts::PI;
use micromath::F32Ext;

pub use robotukfsim::*;     // ﾛﾎｯﾄｼﾐｭﾚｰﾀ
// ----------------------------------------
// 粒子フィルタ次元設定
const PM : usize = 3;       // 状態変数次元
const PC : usize = 2;       // 制御入力次元
const PG : usize = 3200;    // 粒子数
const PD : usize = 2;       // 位置次元
const PN : usize = 2;       // 観測値次元
const LMC: usize = 10;      // ﾗﾝﾄﾞﾏｰｸ数(最大)
// 粒子フィルタパラメータ設定
const SEED: u8   = 43;      // 乱数シード
                            // センサーノイズ
const SENSER_STD_ERR: [f32; PN] = [0.1, 0.1];
                            // 粒子生成ﾊﾟﾗﾒｰﾀ
const CP_PARAM : [[f32; 2]; PM] =
      [[2., 5.], [6., 5.], [0., 2.*PI]];
                            // ﾘｻﾝﾌﾟﾘﾝｸﾞ判定数
const JVAL: f32 = PG as f32 / 2.;
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
// -- 個別定義 --
      rb   : RobotUkfSim,   // ﾛﾎｯﾄｼﾐｭﾚｰﾀ
}
// new
impl SimConfig {
  pub fn new() -> Self {
    // ﾛﾎｯﾄｼﾐｭﾚｰﾀ設定
    let mut rb = RobotUkfSim::new();
    rb.set_random_seed(SEED);
    //
    SimConfig {
      seed : SEED,
      pos  : [rb.sim_pos[0], rb.sim_pos[1]],
      param: CP_PARAM,
      jval : JVAL,
      u    : [0.0; PC],
      dstep: ELLIPSE_STEP,
      lmv  : Vec::<[f32; PD], LMC>::new(),
      zv   : Vec::<[f32; PN], LMC>::new(),
      zp   : [0.0; PD],
      rng  : StdRng::from_seed([SEED; 32]),
      rb,
    }
  }
  // 粒子フィルタ生成
  pub fn particlefilter(&self) 
    -> ParticleFilter<PM, PC, PG, PD, PN>
  {
    let mut pf = ParticleFilter::
                 <PM, PC, PG, PD, PN>::new();
    pf.set_random_seed(self.seed);
    pf.R = SENSER_STD_ERR;
    pf.Q  = [self.rb.sigma_bearing.powi(2),
             self.rb.sigma_range.powi(2)];
    pf.dt = DT;
    // 粒子生成関数
     // 一様分布
/*
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
    // ロボット移動
    self.rb.move_next(DT, &self.u);
    // ロボット位置取得
    self.pos[0] = self.rb.sim_pos[0];
    self.pos[1] = self.rb.sim_pos[1];
    // ﾗﾝﾄﾞﾏｰｸの取得
    self.lmv.clear();
    self.rb.landmarks.iter().for_each(|lm|
      self.lmv.push(*lm).unwrap()
    );
    // 観測値の作成
     // ロボットのﾗﾝﾄﾞﾏｰｸ毎の観測値を取得
    self.zv.clear();
    let mut za = [0.0_f32; PN];
    self.rb.z().chunks(PN).for_each(|z| {
      (0..PN).into_iter().for_each(|c| 
         za[c] = z[c]
      );
      self.zv.push(za).unwrap();
    });
    // 観測位置平均の作成
    self.zp = [0.0; PD];
    self.lmv.iter() // ﾗﾝﾄﾞﾏｰｸ位置[x,y]取得
                    // ﾗﾝﾄﾞﾏｰｸ毎の直距離,向き
      .zip(self.zv.iter())
      .for_each(|(lm, z)| {
         let angle = ad_angl(
           // 観測値の向きは正規化されて
           // いるため、0〜2πに戻す
           z[1] + PI + 
           // ロボットの向きを足してﾗﾝﾄﾞﾏｰｸ
           // からロボットへの角度に変更
           self.rb.sim_pos[2]
         );
         self.zp[0] += lm[0] + 
                       z[0] * angle.cos();
         self.zp[1] += lm[1] +
                       z[0] * angle.sin();
       });
     self.zp[0] /= self.lmv.len() as f32;
     self.zp[1] /= self.lmv.len() as f32;
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
  q  : &[f32],               // ﾌﾟﾛｾｽﾉｲｽﾞ
  dt : f32,                  // dt
  mut rng: &mut StdRng,      // 乱数
)
{
  // 正規分布乱数
  let normal = Normal::new(0.0, 1.0).unwrap();
  let mut randn = || normal.sample(&mut rng);
  // 粒子の移動
  for r in 0..pt.len() {
    pt[r] = RobotUkfSim::move_to(
              &[pt[r][0], pt[r][1], pt[r][2]],
              dt, 
              u
            );
    // 向きにノイズ追加
    pt[r][2] += randn() * q[0];
    pt[r][2]  = ad_angl(pt[r][2]);
    // 位置にノイズ追加
    let dist = randn() * q[1];
    pt[r][0] += pt[r][2].cos() * dist;
    pt[r][1] += pt[r][2].sin() * dist;
  }
}
// === 関数定義型:観測関数ﾃﾞﾌｫﾙﾄ実装 ===
  pub fn hx(
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
    // 向きの要素がある場合ﾗﾝﾄﾞﾏｰｸとの相対角度
    if PM > PD {
      (PD..2*PD-1).into_iter().enumerate()
      .for_each(|(i, c)|
        // ランドマークとの相対角度
        hx[i+1] = RobotUkfSim
                  ::normalize_angle(
                    // 位置との仰角
                    (lm[i+1] - pt[i+1]).atan2
                    (lm[i]   - pt[i]) -
                    // ﾀｰｹﾞｯﾄの角度を引く
                    pt[c]
                  )
      )
    }
    hx
  }

