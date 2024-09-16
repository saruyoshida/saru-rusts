#![no_std]
use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use nalgebra::SMatrix;

#[derive(Clone, Debug)]
pub struct NoisySensor {
  std: f32,
  rng: StdRng,        // 乱数
  normal: Normal<f32>,
}
impl NoisySensor {
  pub fn new(std_noise: f32) -> Self {
    Self {
      std: std_noise,
      rng: StdRng::from_seed([12; 32]),
      normal: Normal::new(0.0, 1.0).unwrap(),
    }
  }
  pub fn sense(
    &mut self,
    pos: (f32, f32),
  ) -> (f32, f32) {
    let mut randn = ||
      self.normal.sample(&mut self.rng);

    (pos.0 + randn() * self.std,
     pos.1 + randn() * self.std)
  }
}
#[derive(Clone, Debug)]
pub struct TurningTarget {
  phi_sim: SMatrix<f32, 4, 4>,
  gam    : SMatrix<f32, 4, 2>,
  x      : SMatrix<f32, 4, 1>,
  turn   : SMatrix<f32, 2, 1>,
  sensor : NoisySensor,
  count  : usize,
  n      : usize,
  turn_start: usize,
}
impl TurningTarget {
  pub fn new(
    n         : usize,
    turn_start: usize,
    dt        : f32,
    std       : f32,
  ) -> Self {
    Self {
      phi_sim : SMatrix::<f32, 4, 4>
                       ::from_column_slice(
                  &[1., 0., 0., 0.,
                    dt, 1., 0., 0.,
                    0., 0., 1., 0.,
                    0., 0., dt, 1.,
                   ]
                ),
      gam     : SMatrix::<f32, 4, 2>
                       ::from_column_slice(
          &[(dt*dt)/2., dt, 0.        , 0.,
            0.        , 0., (dt*dt)/2., dt,
                   ]
                ),
      x       : SMatrix::<f32, 4, 1>
                       ::from_column_slice(
                  &[2000., 0., 10000., -15.]
                ),
      turn    : SMatrix::<f32, 2, 1>
                       ::from_column_slice(
                  &[0.075, 0.075]
                ),
      sensor  : NoisySensor::new(std),
      count   : 0,
      n,
      turn_start,
    }
  }
}
impl Iterator for TurningTarget {
  type Item = [f32; 2];

  fn next(&mut self) -> Option<Self::Item> {
    if self.count > self.n {return None;}

    self.x = self.phi_sim * self.x;
    if self.count >= self.turn_start {
      self.x += self.gam * self.turn;
    }
    self.count += 1;
    let (x, y) =  self.sensor.sense(
                    (self.x[(0, 0)],
                     self.x[(2, 0)])
                  );
    Some([x, y])
  }
}

