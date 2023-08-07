#![no_std]

pub use rand_distr::{Normal, Distribution};
pub use rand::prelude::*;
use micromath::F32Ext;
use one_dimensional::OneDimSimulation;

pub struct NonlinerSimulation {
  x                 : f32, 
  rng               : StdRng,
  normal            : Normal<f32>,
}
// new
impl NonlinerSimulation {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    NonlinerSimulation {
      x : 0.0, 
      rng,
      normal,
    }
  }
}

impl OneDimSimulation for NonlinerSimulation {
// メインメソッド
  // センサー値取得
  fn move_and_sense(&mut self, _ : f32) 
    -> (f32, f32)
  {
    self.x += 1.0;
    (0.0,
     (self.x / 3.0).sin() * 2.0 +
     self.normal.sample(&mut self.rng) *
     1.2
    )
  }
}
impl NonlinerSimulation {
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
