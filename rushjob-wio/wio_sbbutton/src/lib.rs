#![no_std]

use wio_terminal::Button;
use wio_sbeye::WioSBEye;

pub struct WioSBButton {
  cur_button: Button,
  pub eye: WioSBEye,
}

impl WioSBButton {
  pub fn new(radius: i32) -> Self {
    WioSBButton {
      cur_button: Button::TopLeft,
      eye: WioSBEye {
        hw: 0,
        vw: 0,
        r : radius,
        r_limit: 10000,
        step: 5,
      },
    }
  }

  pub fn button_pulled(
    &mut self, 
    button: Button
  ) -> (f32, f32, f32) 
  {
    self.cur_button = button;
    match self.cur_button {
      Button::TopLeft   => 
        self.eye.zoom().position(),
      Button::TopMiddle => 
        self.eye.out().position(),
      Button::Left => 
        self.eye.left().position(),
      Button::Click => 
        self.eye.position(),
      Button::Right => 
        self.eye.right().position(),
      Button::Up => 
        self.eye.up().position(),
      Button::Down => 
        self.eye.down().position(),
    } 
  }

  pub fn position(&self) -> (f32, f32, f32)
  {  self.eye.position()  }
}


