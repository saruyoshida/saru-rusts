#![no_std]

use discrete_bayes::DiscreteBayes;
// use robot::Robot; //**削除


pub struct ObjObsFilter {
  track          : [i32; TLEN], // 経路
  kernel         : [f32; KLEN], // カーネル
  sensor_accuracy: f32,       // センサ精度
  prior          : [f32; TLEN], // 事前分布
  posterior      : [f32; TLEN], // 事後分布
  likelihood     : [f32; TLEN], // 尤度
  move_distance  : i32,         // 移動量
  target         : usize,//**変更
  // 離散ベイズ関数達
  db             : DiscreteBayes,
  loop_cnt       : i32,         // 繰返し回数
  dsp_cnt        : i32,         // 表示開始数
  measurements   : [i32; TLEN],//**追加
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
    let track = [1, 0, 1, 0, 0, 1, 0, 1, 0, 
                 0];//**変更
    // 不正確なセンサデータ
    let measurements =  
                [1, 0, 1, 0, 0, 1, 1, 1, 0, 
                 0];//**追加
    // カーネル
    let kernel          = [0.1, 0.8, 0.1];
    // 指示移動量
    let move_distance   = 1;//**変更
    // 事前分布
    let mut prior       = [0.1; TLEN];//**変更
//    prior[0]            = 0.9;//**削除
    // 事後分布
    let mut posterior   = prior.clone();
    // 尤度
    let likelihood      = [1.0; TLEN];
    // センサ精度
    let sensor_accuracy = 0.75;//**変更
    // 乱数シード
//    let random_seed     = 4_u64;//**削除
    // 繰返し回数
    let loop_cnt        = TLEN as i32;//**変更
    // 表示開始回数
    let dsp_cnt         = 0;
    //------------------------------------
    // 離散ベイズ関数達
    let db = DiscreteBayes;
    // 事前分布、事後分布の正規化
    db.normalize(&mut prior);
    db.normalize(&mut posterior);
    // ロボット
//    let target = Robot::new(        //**削除
//      TLEN,            // 経路数
//      sensor_accuracy, // センサ精度
//      random_seed,     // シード
//    );
    let target = 0;
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
      measurements,
    }
  }
}
// フィルタ操作メイン
impl ObjObsFilter {
  pub fn iterations(&mut self) {
    // ターゲットを移動量分動かす
//    self.target.move_to( //**削除
//      self.move_distance,
//      &self.kernel,
//    );
//
    // 事後分布、移動量、カーネルから
    // 事前分布を予測
    self.db.predict(
      &self.posterior,      // 事後分布
      self.move_distance,   // 移動量
      &self.kernel,         // カーネル
      &mut self.prior,      // 事前分布
    );
    // ターゲットのセンサから観測値を取得
    let m = self.measurements[self.target];
    //**変更

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
    self.target += 1;//**追加
  }
}
// ゲッター
impl ObjObsFilter {
// 削除
//  // 事後分布の最大値とそのインデックス取得
//  pub fn argmax(&self) -> (usize, f32) {
//    self.db.argmax(
//      &self.posterior
//    )
//  }
//
//  // ターゲットのポジション
//  pub fn pos(&self) -> i32 {
//    self.target.pos()
//  }
//
  // ターゲットの観測値
  pub fn sensor_pos(&self) -> i32 {
    self.measurements[self.target - 1_usize]
    //**変更
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
    