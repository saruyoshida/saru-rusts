#![no_std]

pub use rand_distr::{Normal, Distribution};
pub use rand::prelude::*;
use micromath::F32Ext;
use one_dimensional::OneDimSimulation;

pub struct DogSimulation {
  x                 : f32, // 位置
  velocity          : f32, // 移動量
  measurement_noise : f32, // 測定ノイズ
  process_noise     : f32, // 動作ノイズ
  add_velocity      : f32, // 加速値
  rng               : StdRng,
  normal            : Normal<f32>,
}
// new
impl DogSimulation {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    DogSimulation {
      x                : 0.0, // 初期位置
      velocity         : 0.0, // 移動量
      measurement_noise: 0.0, // 測定値ノイズ
      process_noise    : 0.0, // 動作値ノイズ
      add_velocity     : 0.0, // 加速値
      rng,
      normal,
    }
  }
}

impl OneDimSimulation for DogSimulation {
// メインメソッド
  // move
  fn move_to(
    &mut self, 
    dt : f32,
  )
  {
    self.velocity += self.add_velocity;

    let velocity = self.velocity + 
      self.normal.sample(&mut self.rng) *
      self.process_noise * dt;

    self.x += velocity * dt;
  }
  // センサー値取得
  fn sense_position(&mut self) -> f32 {
    self.x + 
    self.normal.sample(&mut self.rng) *
    self.measurement_noise
  }
  // ムーブしてセンサー値取得
  fn move_and_sense(
    &mut self,
    dt : f32
  ) -> (f32, f32)
  {
    self.move_to(dt);
    (self.x, self.sense_position())
  }
// ゲッター
  fn x(&self) -> f32 {
    self.x
  }
}
// セッター
impl DogSimulation {
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
  // 位置
  pub fn set_x(&mut self, x : f32)
    -> &mut Self {
    self.x = x;
    self
  }
  // 移動量
  pub fn set_velocity(
    &mut self, 
    velocity : f32
  ) -> &mut Self {
    self.velocity = velocity;
    self
  }
  // 移動量加速
  pub fn set_add_velocity(
    &mut self, 
    velocity : f32
  ) -> &mut Self {
    self.add_velocity = velocity;
    self
  }
  // 測定値分散
  pub fn set_measurement_var(
    &mut self, 
    measurement_var : f32
  ) -> &mut Self {
    self.measurement_noise =
                    measurement_var.sqrt();
    self
  }
  // 動作値分散
  pub fn set_process_var(
    &mut self, 
    process_var : f32
  ) -> &mut Self {
    self.process_noise = process_var.sqrt();
    self
  }


}
