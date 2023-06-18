#![no_std]

use oorandom::Rand32;

pub struct Robot {
  track_len      : i32,
  pos            : i32,
  sensor_pos     : i32,
  sensor_accuracy: f32,
  rand           : Rand32,
}
// new
impl Robot {
  pub fn new(
    track_len      : usize,     // 経路数
    sensor_accuracy: f32,       // センサ精度
    random_seed    : u64,       // シード
  ) -> Self 
  {
    Robot {
      track_len : track_len as i32,
      pos       : 0,
      sensor_pos: 0,
      sensor_accuracy,
      rand: Rand32::new(random_seed),
    }
  }
}

impl Robot {
  pub fn move_to(
    &mut self, 
    distance: i32,
    kernel  : &[f32],
  ) -> i32
  {
    self.pos += distance;
    // カーネルに従うような誤差を加える
    let r = self.rand.rand_float();

    let mut s = 0.0;
    let mut offset = 
      -(kernel.len() as i32 - 1) / 2;

    for k in kernel {
      s += k;
      if r <= s {break;}
      offset += 1;
    }

    self.pos = (self.pos + offset)  %
               self.track_len;
    if self.pos < 0 {
      self.pos += self.track_len;
    }

    self.pos
  }

  pub fn sense(&mut self) -> i32 {
    self.sensor_pos = self.pos;
    // センサーの誤差を加える
    if self.rand.rand_float() >
      self.sensor_accuracy 
    {
      if self.rand.rand_float() > 0.5 {
        self.sensor_pos += 1
       } else {
        self.sensor_pos -= 1
       }
    }
    self.sensor_pos
  }
}
// セッター
impl Robot {
  pub fn set_rand(
    &mut self,
    random_seed: u64
  )
  {
    self.rand = Rand32::new(random_seed);
  }
}
// ゲッター
impl Robot {
  pub fn pos(&self) -> i32 {
    self.pos
  }

  pub fn sensor_pos(&self) -> i32 {
    self.sensor_pos
  }
}
