#![no_std]

pub use rand_distr::{Normal, Distribution};
pub use rand::prelude::*;

pub struct Robot2d {
  pub pos       : (f32, f32), // 位置
  pub vel       : (f32, f32), // 速度
  pub noise_std : f32,        // ノイズ
  rng           : StdRng,
  normal        : Normal<f32>,
}
// new
impl Robot2d {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    Robot2d {
      pos       : (0.0, 0.0), // 位置
      vel       : (1.0, 1.0), // 速度
      noise_std : 1.0,        // ノイズ
      rng,
      normal,
    }
  }
}

impl Robot2d {
// メインメソッド
  pub fn read(&mut self) -> [f32; 2]
  {
    self.pos.0 += self.vel.0;
    self.pos.1 += self.vel.1;

    [self.pos.0 + 
     self.normal.sample(&mut self.rng) *
     self.noise_std,
     self.pos.1 + 
     self.normal.sample(&mut self.rng) *
     self.noise_std
    ]
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
