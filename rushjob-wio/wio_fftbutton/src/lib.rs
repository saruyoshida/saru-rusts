#![no_std]

use wio_terminal::Button;

pub struct WioFftButton {
  topleft: (f32, f32),
  topmiddle: (f32, f32),
  // Topright,
  down: (f32, f32),
  up: (f32, f32),
  left: (f32, f32),
  right: (f32, f32),
  click: (f32, f32),
}

impl WioFftButton {
  pub fn new(
    topleft:f32,
    topmiddle: f32,
    // Topright,
    down: f32,
    up: f32,
    left: f32,
    right: f32,
    click: f32,
    fs_range: f32,
  ) -> Self 
  {
    WioFftButton {
      topleft:   (topleft - fs_range,
                  topleft + fs_range),
      topmiddle: (topmiddle - fs_range,
                  topmiddle + fs_range),
      down:      (down - fs_range,
                  down + fs_range),
      up:        (up - fs_range,
                  up + fs_range),
      left:      (left - fs_range,
                  left + fs_range),
      right:     (right - fs_range,
                  right + fs_range),
      click:     (click - fs_range,
                  click + fs_range),
    }
  }

  pub fn get_button(&mut self, fs: f32)
    -> Option<Button>
  {
    match fs {
      x if x > self.topleft.0 &&
           x < self.topleft.1 =>
           Some(Button::TopLeft),
      x if x > self.topmiddle.0 &&
           x < self.topmiddle.1 =>
           Some(Button::TopMiddle),
      x if x > self.left.0 &&
           x < self.left.1 =>
           Some(Button::Left),
      x if x > self.click.0 &&
           x < self.click.1 =>
           Some(Button::Click),
      x if x > self.right.0 &&
           x < self.right.1 =>
           Some(Button::Right),
      x if x > self.up.0 &&
           x < self.up.1 =>
           Some(Button::Up),
      x if x > self.down.0 &&
           x < self.down.1 =>
           Some(Button::Down),
      _ => None
    }
  }
}








