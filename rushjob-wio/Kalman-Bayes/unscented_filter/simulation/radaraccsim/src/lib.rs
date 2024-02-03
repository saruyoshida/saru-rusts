#![no_std]

use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use micromath::F32Ext;

const N:usize = 2;

pub struct ACSim {
  pub pos     : [f32; N], // 位置,高度
  pub vel     : [f32; N], // 速度m/s,上昇率
  pub vel_std : f32,       // ノイズ
  pub dt      : f32,
  rng         : StdRng,
  normal      : Normal<f32>,
}
// new
impl ACSim {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    ACSim {
      pos     : [0.; N], // 位置,高度
      vel     : [0.; N], // 速度m/s,上昇率
      vel_std : 0.,      // ノイズ
      dt      : 0.,
      rng,
      normal,
    }
  }
}

impl ACSim {
  pub fn update(&mut self) -> [f32; N]
  {
    let mut randn = ||
      self.normal.sample(&mut self.rng);
   
    self.vel.iter()
        .map(|v| v * self.dt +
                 (randn() * self.vel_std) *
                 self.dt)
        .enumerate()
        .for_each(|(i, p)| self.pos[i] += p);
               
    self.pos
  }
  pub fn vxy(
    &mut self,
    vel_std: f32
  ) -> [f32; N]
  {
    let mut randn = ||
      self.normal.sample(&mut self.rng);
    
    let mut vel = [0.; N];
    self.vel.iter()
        .map(|v| v + randn() * vel_std)
        .enumerate()
        .for_each(|(i, p)| vel[i] = p);
    vel
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

pub struct RadarStation {
  pub pos            : [f32; N],
  pub range_std      : f32,
  pub elev_angle_std : f32,
  rng                : StdRng,
  normal             : Normal<f32>,
}

impl RadarStation {
  pub fn new() -> Self {
    let rng = StdRng::from_seed(
                [42; 32]
              );
    let normal = Normal::new(0.0, 1.0)
                 .unwrap();

    RadarStation {
      pos            : [0.; N], 
      range_std      : 0., 
      elev_angle_std : 0.,     
      rng,
      normal,
    }
  }
//(直距離, 仰角) を返す。
// 仰角の単位はラジアン。
  pub fn reading_of(
    &self, 
    ac_pos: [f32; N]
  ) -> [f32; N]
  {
    let mut diff = [0.; N];
    ac_pos.iter()
          .zip(self.pos.iter())
          .map(|(a, s)| a - s)
          .enumerate()
          .for_each(|(i, p)| diff[i] = p);
    // 直距離norm:√a[0]**2 + a[1]**2 + ..
    [diff.iter().map(|a| a.powi(2))
                .sum::<f32>()
                .sqrt(),
    // 仰角
     diff[1].atan2(diff[0])
    ]
  }
// シミュレートされたノイズを持った 
// 直距離と仰角の観測値を返す
  pub fn noisy_reading(
    &mut self, 
    ac_pos: [f32; N]
  ) -> [f32; N]
  { 
    // 航空機の直距離と仰角
    let mut r = self.reading_of(ac_pos);
    // ノイズ追加
    let mut randn = ||
      self.normal.sample(&mut self.rng);

    [self.range_std, self.elev_angle_std]
    .iter()
    .map(|s| s * randn())
    .enumerate()
    .for_each(|(i, p)| r[i] += p);

    r
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
