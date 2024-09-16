#![no_std]
use rand_distr::{Normal, Distribution};
use rand::prelude::*;
use micromath::F32Ext;

fn angle_between(x: f32, y: f32) -> f32 {
  [(y-x).abs(),
   (y-x+360.).abs(),
   (y-x-360.).abs(),
  ].into_iter().fold(0./0., f32::min)
}
#[derive(Clone, Debug)]
pub struct ManeuveringTarget {
  x  : f32,
  y  : f32,
  vel: f32,
  hdg: f32,
  cmd_vel : f32,
  cmd_hdg : f32,
  vel_step: f32,
  hdg_step: f32,
  vel_delta: f32,
  hdg_delta: f32,
}
impl ManeuveringTarget {
  pub fn new(
    x: f32,
    y: f32,
    vel: f32,
    hdg: f32,
  ) -> Self {
    Self {
      x,
      y,
      vel,
      hdg,
      cmd_vel : vel,
      cmd_hdg : hdg,
      vel_step: 0.,
      hdg_step: 0.,
      vel_delta: 0.,
      hdg_delta: 0.,
    }
  }
  pub fn update(&mut self) -> (f32, f32) {
    let vx = self.vel * 
      (90.-self.hdg).to_radians().cos();
    let vy = self.vel * 
      (90.-self.hdg).to_radians().sin();

    self.x += vx;
    self.y += vy;

    if self.hdg_step > 0. {
      self.hdg_step -= 1.;
      self.hdg += self.hdg_delta;
    }

    if self.vel_step > 0. {
      self.vel_step -= 1.;
      self.vel += self.vel_delta;
    }
    (self.x, self.y)
  }
  pub fn set_commanded_heading(
    &mut self, 
    hdg_degrees: f32,
    steps      : f32,
  )
  {
    self.cmd_hdg = hdg_degrees;
    self.hdg_delta = angle_between(
      self.cmd_hdg,
      self.hdg
    ) / steps;
    if self.hdg_delta.abs() > 0. {
      self.hdg_step = steps;
    } else {
      self.hdg_step = 0.;
    }
  }
  pub fn set_commanded_speed(
    &mut self,
    speed: f32,
    steps: f32,
  )
  {
    self.cmd_vel = speed;
    self.vel_delta = (self.cmd_vel - 
                      self.vel) / steps;
    if self.vel_delta.abs() > 0. {
      self.vel_step = steps;
    } else {
      self.vel_step = 0.
    }
  }
}
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
pub struct GenerateData {
  steady_count: usize,
  count       : usize,
  t           : ManeuveringTarget,
  n           : NoisySensor,
}
impl GenerateData {
  pub fn new(
    steady_count: usize,
    std         : f32
  ) -> Self {
    Self {
      steady_count,
      count: 0,
      t    : ManeuveringTarget::new(
               0.0, 0.0, 0.3, 0.0
             ),
      n    : NoisySensor::new(std),
    }
  }
}
impl Iterator for GenerateData {
  type Item = ((f32, f32), (f32, f32));

  fn next(&mut self) -> Option<Self::Item> {
    if self.count > 30 + self.steady_count {
      return None;
    }
    if self.count == 30 {
      self.t.set_commanded_heading(310., 25.);
      self.t.set_commanded_speed(1., 15.);
    }
    self.t.update();
    self.count += 1;
    Some((
      (self.t.x, self.t.y),
      self.n.sense((self.t.x, self.t.y))
    ))
  }
}

