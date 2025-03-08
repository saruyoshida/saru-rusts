#![no_std]

#[allow(unused_imports)]
use micromath::F32Ext;

// 作用積分
pub struct ActionIntegral {
  xt        : TrailFunction, // 試行関数
  lagrangian: Oscillator,    // ﾗｸﾞﾗﾝｼﾞｱﾝ
  diff      : Differential,  // 数値微分演算子
  integral  : Integral,      // 数値積分演算子
}
impl ActionIntegral { 
  // 変分要素更新
  pub fn set_k(&mut self, k: f32) -> &Self {
    self.xt.set_k(k);
    self
  }
  // 作用積分
  pub fn action_integral(&self) -> f32 {
    self.integral.int_t(self)
  }
  // ラグランジアン: L(x(t), x'(t))
  pub fn lagrangian(&self, t: f32) -> f32 {
    self.lagrangian.call(
      self.xt.call(t), 
      self.diff.d_t(&self.xt, t)
    )
  }
}
// 数値微分演算子
pub struct Differential { dt: f32 }
impl Differential {
  pub fn d_t(
    &self, 
    func_t: &TrailFunction,  // 対象関数
    t: f32,
  ) -> f32 
  {
    // 中心差分: (f(x+h)-f(x-h))/2h
    (func_t.call(t + 0.5 * self.dt) - 
     func_t.call(t - 0.5 * self.dt)
    ) / self.dt
  }
}
// 数値定積分演算子
pub struct Integral {
  t0: f32,     // 境界条件(始端)
  t1: f32,     // 境界条件(終端)
  ndiv: usize, // 分割数
}
impl Integral {
  pub fn int_t(
    &self,
    action_integral: &ActionIntegral
  ) -> f32 {
    // ∫t0->t1 L(x(t), x'(t))
    let dt = (self.t1 - self.t0) 
             / self.ndiv as f32;
    let mut t = self.t0 + 0.5 * dt;
    let mut s = 0.0;
    (0..self.ndiv).for_each(|_| {
      s += action_integral.lagrangian(t);
      t += dt;
    });
    s * dt
  }
}

// 振動子のラグランジアン
#[allow(non_snake_case)]
pub struct Oscillator { M: f32, K: f32 }
impl Oscillator {
  pub fn call(&self, x: f32, v: f32) -> f32 {
    0.5 * (self.M*v*v - self.K*x*x)
  }
}
// 試行関数
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct TrailFunction {
  k:  f32, // 変分要素
  t1: f32, // 境界条件(終端)
}
impl TrailFunction {
  // 変分要素更新
  pub fn set_k(&mut self, k: f32) { 
    self.k = k;
  }
  // 関数実行
  pub fn call(&self, t: f32) -> f32 {
    (self.k*t).sin() / (self.k*self.t1).sin()
  }
}

#[allow(non_snake_case)]
pub fn create_trial_action(
  W: f32,
) -> ActionIntegral
{
  let t0 = 0.0;   // 境界条件(始端)
  let t1 = 1.0;   // 境界条件(終端)
  let ndiv = 200; // 分割数
  // 作用積分構造体作成
  ActionIntegral {
    xt        : TrailFunction { k: 0.0, t1 },
    lagrangian: Oscillator { M: 1.0, K: W*W },
    diff      : Differential { dt: 0.0001 },
    integral  : Integral { t0, t1, ndiv },
  }
}

