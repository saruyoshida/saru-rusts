#![no_std]

use core::f32::consts::PI;
use micromath::F32Ext;

pub struct WioSBEye {
  pub hw     : i32,
  pub vw     : i32,
  pub r      : i32,
  pub r_limit: i32,
  pub step   : i32
}

impl WioSBEye {
  pub fn left(&mut self) -> &mut Self {
    self.hw = Self::gatch(self.hw,-self.step);
    self
  }

  pub fn right(&mut self) -> &mut Self {
    self.hw = Self::gatch(self.hw, self.step);
    self
  }
    
  pub fn up(&mut self) -> &mut Self {
    self.vw = Self::gatcv(self.vw, self.step);
    self
  }
  
  pub fn down(&mut self) -> &mut Self {
    self.vw = Self::gatcv(self.vw,-self.step);
    self
  }

  pub fn zoom(&mut self) -> &mut Self {
    if (self.r - self.step) > -self.r_limit {
        self.r -= self.step;
    }
    self
  }

  pub fn out(&mut self) -> &mut Self {
    if (self.r + self.step) < self.r_limit {
        self.r += self.step;
    }
    self
  }

  pub fn position(&self) 
    -> (f32, f32, f32)
  {
    let ph = self.vw as f32 / 180.0 * PI;
    let th = self.hw as f32 / 180.0 * PI;
    let r  = self.r  as f32;

    (r * ph.cos() * th.sin(),
     r * ph.sin() * th.cos().abs(), 
     r * ph.cos() * th.cos(),
    )
  }

  fn gatch(w: i32, step: i32) -> i32 
  { 
    if (w + step) >= 360
    {
      w + step - 360
    } else if (w + step) < 0
    {
      w + step + 360
    } else
    {
      w + step
    }
  }

  fn gatcv(w: i32, step: i32) -> i32 
  { 
    if (w + step) <  90 &&
       (w + step) > -90
    {
      w + step
    } else
    {
      w 
    } 
  }
}
