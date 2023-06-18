#![no_std]

use discrete_bayes::DiscreteBayes;
use robot::Robot;

pub struct ObjObsFilter {
  track          : [i32; TLEN], // 経路
  kernel         : [f32; KLEN], // カーネル
  sensor_accuracy: f32,         // センサ精度
  prior          : [f32; TLEN], // 事前分布
  posterior      : [f32; TLEN], // 事後分布
  likelihood     : [f32; TLEN], // 尤度
  move_distance  : i32,         // 移動量
  target         : Robot,       // ロボット
  // 離散ベイズ関数達
  db             : DiscreteBayes,
  loop_cnt       : i32,         // 繰返し回数
  dsp_cnt        : i32,         // 表示開始数
}

// 初期設定
const TLEN: usize = 10; // トラック長
const KLEN: usize = 3;  // カーネル長
// ビルド
impl ObjObsFilter {
  pub fn new() -> Self
  {
    // フィルタ設定値 ---------------------
    // 経路
    let track = [0, 1, 2, 3, 4, 5, 6, 7, 8, 
                 9];
    // カーネル
    let kernel          = [0.1, 0.8, 0.1];
    // 指示移動量
    let move_distance   = 4;
    // 事前分布
    let mut prior       = [0.01; TLEN];
    prior[0]            = 0.9;
    // 事後分布
    let mut posterior   = prior.clone();
    // 尤度
    let likelihood      = [1.0; TLEN];
    // センサ精度
    let sensor_accuracy = 0.9;
    // 乱数シード
    let random_seed     = 4_u64;
    // 繰返し回数
    let loop_cnt        = 150;
    // 表示開始回数
    let dsp_cnt         = 0;
    //------------------------------------
    // 離散ベイズ関数達
    let db = DiscreteBayes;
    // 事前分布、事後分布の正規化
    db.normalize(&mut prior);
    db.normalize(&mut posterior);
    // ロボット
    let target = Robot::new(
      TLEN,            // 経路数
      sensor_accuracy, // センサ精度
      random_seed,     // シード
    );
    // フィルタ
    ObjObsFilter {
      track,
      kernel,
      sensor_accuracy,
      prior,
      posterior,
      likelihood,
      move_distance,
      target,
      db,
      loop_cnt,
      dsp_cnt,
    }
  }
}
// フィルタ操作メイン
impl ObjObsFilter {
  pub fn iterations(&mut self) {
    // ターゲットを移動量分動かす
    self.target.move_to(
      self.move_distance,
      &self.kernel,
    );
    // 事後分布、移動量、カーネルから
    // 事前分布を予測
    self.db.predict(
      &self.posterior,      // 事後分布
      self.move_distance,   // 移動量
      &self.kernel,         // カーネル
      &mut self.prior,      // 事前分布
    );
    // ターゲットのセンサから観測値を取得
    let m = self.target.sense();
    // 観測値、センサ誤差から
    // 尤度（もっともらしさ）を計算
    self.db.lh_hallway(
      &self.track,          // 経路 
      m,                    // 観測値
      self.sensor_accuracy, // センサ精度
      &mut self.likelihood, // 尤度
    );
    // 尤度、事前分布から事後分布を計算・更新
    self.db.update(
      &self.likelihood,     // 尤度
      &self.prior,          // 事前分布
      &mut self.posterior   // 事後分布
    );
  }
}
// ゲッター
impl ObjObsFilter {
  // 事後分布の最大値とそのインデックス取得
  pub fn argmax(&self) -> (usize, f32) {
    self.db.argmax(
      &self.posterior
    )
  }
  // ターゲットのポジション
  pub fn pos(&self) -> i32 {
    self.target.pos()
  }
  // ターゲットの観測値
  pub fn sensor_pos(&self) -> i32 {
    self.target.sensor_pos()
  }
  // 事前分布
  pub fn prior(&self) -> &[f32] {
    self.prior.as_slice()
  }
  // 事後分布
  pub fn posterior(&self) -> &[f32] {
    self.posterior.as_slice()
  }
  // 繰返し回数
  pub fn loop_cnt(&self) -> i32 {
    self.loop_cnt
  }
  // 表示開始回数
  pub fn dsp_cnt(&self) -> i32 {
    self.dsp_cnt
  }
}
    