#![no_std]

use rand_distr::{Normal, Distribution};
use rand::prelude::*;

pub struct ConstantAcc {
  pub x          : f32, // 位置
  pub vel        : f32, // 速度
  pub acc        : f32, // 加速度
  pub noise_scale: f32, // ノイズ
  rng            : StdRng,
  normal         : Normal<f32>,
}
// new
impl ConstantAcc {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    ConstantAcc {
      x          : 0.0, // 位置
      vel        : 1.0, // 速度
      acc        : 0.1, // 加速度
      noise_scale: 0.1, // ノイズ
      rng,
      normal,
    }
  }
}

impl ConstantAcc {
// メインメソッド
  pub fn read(&mut self) -> [f32; 3]
  {
    self.acc += 
      self.normal.sample(&mut self.rng) *
      self.noise_scale;
    self.vel += self.acc;
    self.x   += self.vel;

    [self.x, self.vel, self.acc]
  }
  // センサ
  pub fn sense(
    &mut self,
    x          : &[f32],
    noise_scale: f32,
  ) -> f32
  {
    x[0] +
    self.normal.sample(&mut self.rng) *
    noise_scale
  }
// セッター
  // 乱数シード
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
