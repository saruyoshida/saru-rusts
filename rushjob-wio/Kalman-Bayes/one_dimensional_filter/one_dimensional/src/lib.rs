#![no_std]

// mean,mu      : 平均
// median       : 中央値
//                {1.8,2.0,1.7,1.9,1.6}の集合
//                をソートして3番目にくる値
// X            : 確率変数
// E[X]         : Xの期待値
//                標本空間[1,3,5]で
//                1が80%,3が15%,5が5%の場合
//                1*0.8+3*0.15+5*0.05=1.5
// var          : 分散 E[(X-mu)^2)]
//                確率変数-平均の二乗の期待値
// std          : 標準偏差(√var)
// gaussian     : ガウス分布(正規分布)
// gaussian_pdf : ガウス分布の確率密度関数
// 1σ,2σ,,      : 標準偏差1つ分、2つ分、、
// 

// ガウス分布
#[derive(Debug, PartialEq, Clone)]
pub struct Gaussian {
  pub mean : f32,        // 平均
  pub var  : f32,        // 分散
}
impl Gaussian {
  pub fn mean_var(&self) -> (f32, f32) {
    (self.mean, self.var)
  }
}

// ガウス分布の積(尤度x事前分布)パターン
pub struct OneDimKalman;
// 
impl OneDimFilter for OneDimKalman {
  // 予測
  fn predict(
    &self,
    posterior: &Gaussian,
    movement : &Gaussian,
  ) -> Gaussian
  {
    Gaussian {
      mean: posterior.mean + movement.mean, 
      var : posterior.var  + movement.var,
    }
  }
  // 更新
  fn update(
    &self,
    prior      : &Gaussian,
    measurement: &Gaussian,
  ) -> (Gaussian, f32)
  {
    self.gaussian_multiply(measurement, prior)
  }
}
impl OneDimKalman {
  // ガウス分布の積
  pub fn gaussian_multiply(
    &self,
    g1     : &Gaussian,
    g2     : &Gaussian,
  ) -> (Gaussian, f32)
  {
    let mean =
      (g1.var * g2.mean + g2.var * g1.mean) / 
      (g1.var + g2.var);

    let variance = (g1.var * g2.var) / 
                   (g1.var + g2.var);
   
    (Gaussian {mean, var: variance},
     g1.var / (g1.var + g2.var)     
    )
  }
}
// カルマンゲインパターン
pub struct OneDimKalmanRest;
// x     : 事前分布の平均
// P     : 事前分布の分散
// z     : 観測値の平均
// R     : 観測値の分散
// K     : カルマンゲイン
//         = P / (P + R)
impl OneDimFilter for OneDimKalmanRest {
  // 予測
  fn predict(
    &self,
    posterior : &Gaussian,
    movement  : &Gaussian,
  ) -> Gaussian
  {
    #[allow(non_snake_case)]
    let (mut x, mut P) = posterior.mean_var();

    #[allow(non_snake_case)]
    let (dx, Q)        = movement.mean_var();

    x = x + dx;
    P = P + Q;

    Gaussian {mean: x, var: P}
  }
  // 更新
  fn update(
    &self,
    prior       : &Gaussian,
    measurement : &Gaussian,
  ) -> (Gaussian, f32)
  {
    #[allow(non_snake_case)]
    let (mut x, mut P) = prior.mean_var();

    #[allow(non_snake_case)]
    let (z, R)      = measurement.mean_var();

    let y = z - x;

    #[allow(non_snake_case)]
    let K = P / (P + R);

    x = x + K * y;
    P = (1.0 - K) * P;

    (Gaussian{mean: x, var: P}, K)
  }
}
// 固定ゲインパターン
pub struct OneDimKalmanFix{
  k : f32
}
// x     : 事前分布の平均
// z     : 観測値の平均
// K     : 固定ゲイン
impl OneDimFilter for OneDimKalmanFix {
  // 予測
  fn predict(
    &self,
    posterior : &Gaussian,
    movement  : &Gaussian,
  ) -> Gaussian
  {
    let (x , _) = posterior.mean_var();
    let (dx, _) = movement.mean_var();

    Gaussian {mean: x + dx, var: 0.0}
  }
  // 更新
  fn update(
    &self,
    prior       : &Gaussian,
    measurement : &Gaussian,
  ) -> (Gaussian, f32)
  {
    let (x, _) = prior.mean_var();
    let (z, _) = measurement.mean_var();

    let y = z - x;   // 残差
    
    (Gaussian {mean: x + self.k * y, var: 0.0}
     , self.k)
  }
}

// フィルタトレイト
pub trait OneDimFilter {
  // 予測
  fn predict (
    &self,
    posterior : &Gaussian,
    movement  : &Gaussian,
  ) -> Gaussian;
  // 更新
  fn update (
    &self,
    prior       : &Gaussian,
    measurement : &Gaussian,
  ) -> (Gaussian, f32);
}
// シミュレータトレイト
pub trait OneDimSimulation {
  // move
  fn move_to(&mut self, _dt : f32){}
  // センサー値取得
  fn sense_position(&mut self) -> f32 {0.0}
  // moveしてセンサー値取得
  fn move_and_sense(
    &mut self,
    _dt : f32
  ) -> (f32, f32) {(0.0, 0.0)}
// ゲッター
  // ターゲット位置
  fn x(&self) -> f32 {0.0}
}