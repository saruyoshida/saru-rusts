#![no_std]

use one_dimensional::*;

pub struct ObjObsFilter<T, F> 
{
  prior          : Gaussian, // 事前分布
  x              : Gaussian, // 事後分布
  z              : f32,      // 観測値
  process_model  : Gaussian, // プロセスモデル
  sensor_var     : f32,      // センサーの分散
  dt             : f32,      // タイムステップ 
  target         : T,      // シミュレーション
  kfn            : F,      // フィルタ関数
  k              : f32,    // カルマンゲイン
  init_ztox      : bool,   // 初期観測値を
                           // 初期推測値にする
}
// ビルド
impl<T, F> ObjObsFilter<T, F> 
    where F : OneDimFilter,
          T : OneDimSimulation,
{
  pub fn new(
    target : T,
    kfn    : F,
  ) -> Self
  {
    // フィルタ初期値 ---------------------
    let init_gaussian = Gaussian{
                          mean : 0.0,
                          var  : 0.0,
                        };
    //------------------------------------
    // フィルタ
    ObjObsFilter {
      prior         : init_gaussian.clone(),
      x             : init_gaussian.clone(),
      z             : 0.0,
      process_model : init_gaussian.clone(),
      sensor_var    : 0.0,
      dt            : 0.0,
      target,
      kfn,
      k             : 0.0,
      init_ztox     : false,
    }
  }
}
// フィルタ操作メイン
impl<T, F> ObjObsFilter<T, F> 
    where F : OneDimFilter,
          T : OneDimSimulation,
{
  pub fn iterations(&mut self) {
    // シミュレーションから観測値を取得
    self.z = self.target.move_and_sense(
               self.dt
             ).1;
    // 初期観測値を初期推定値とするなら
    if self.init_ztox {
      self.x.mean = self.z;
      self.init_ztox = false;
    }
    // 事後分布、プロセスモデルから
    // 事前分布を予測
    self.prior = self.kfn.predict(
      &self.x,              // 事後分布
      &self.process_model,  // プロセスモデル
    );
    // 観測値、センサ誤差から
    // 尤度（もっともらしさ）を設定
    let likelihood = Gaussian {
      mean : self.z,          // 観測値
      var  : self.sensor_var, // センサー分散
    };
    // 尤度、事前分布から事後分布を計算・更新
    (self.x, self.k) = self.kfn.update(
      &self.prior,          // 事前分布
      &likelihood,          // 尤度
    );
  }
}
// セッター
impl<T, F> ObjObsFilter<T, F> 
    where F : OneDimFilter,
          T : OneDimSimulation,
{
  // プロセスモデル(平均、分散)
  pub fn set_process_model(
    &mut self,
    c : (f32, f32)
  ) -> &mut Self
  {
    self.process_model.mean = c.0;
    self.process_model.var  = c.1;
    self
  }
  // センサーの分散
  pub fn set_sensor_var(
    &mut self,
    c : f32,
  ) -> &mut Self
  {
    self.sensor_var = c;
    self
  }
  // 初期推定位置,分散
  pub fn set_x(
    &mut self,
    c : (f32, f32),
  ) -> &mut Self
  {
    self.x.mean = c.0;
    self.x.var  = c.1;
    self
  }
  // 初期位置設定
  pub fn set_init_x(
    &mut self,
    c : f32,
  ) -> &mut Self
  {
    self.x.mean = c;
    self
  }
  // タイムステップ
  pub fn set_dt(
    &mut self,
    c : f32,
  ) -> &mut Self
  {
    self.dt = c;
    self
  }
  // 初期観測値を初期推定値とする
  pub fn set_init_ztox(&mut self) -> &mut Self
  {
    self.init_ztox = true;
    self
  }
}
// ゲッター
impl<T, F> ObjObsFilter<T, F> 
    where F : OneDimFilter,
          T : OneDimSimulation,
{
  // ターゲットの観測値
  pub fn zs(&self) -> f32 {
    self.z
  }
  // ターゲットの実際位置
  pub fn target_x(&self) -> f32 {
    self.target.x()
  }
  // 事前分布
  pub fn prior(&self) -> (f32, f32) {
    self.prior.mean_var()
  }
  // 事後分布
  pub fn xs(&self) -> (f32, f32) {
    self.x.mean_var()
  }
  // カルマンゲイン
  pub fn k(&self) -> f32 {
    self.k
  }
}
    