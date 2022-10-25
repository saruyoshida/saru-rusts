#![no_std]

use embedded_graphics::{
mono_font::{ascii::FONT_9X15_BOLD,
    MonoTextStyle},
  pixelcolor::Rgb565,
  prelude::*,
  primitives::{PrimitiveStyle, Rectangle},
  text::Text,
};

use heapless::consts::U256;
use heapless::String;

pub struct WioToast {
  limit_count: u32,
  limit: u32,
  start_point: Point,
  box_size: Size,
  back_color: Rgb565,
  reset_color: Rgb565,
  text_color: Rgb565,
  toast_text: String::<U256>,
}

impl WioToast {
  pub fn new(
    limit: u32,
    start_point: Point,
    box_size: Size,
    back_color: Rgb565,
    reset_color: Rgb565,
    text_color: Rgb565,
    ) -> Self 
  {
    let toast_text = String::<U256>::new();
    let limit_count: u32 = 0;

    let wio_toast = WioToast {
      limit_count,
      limit,
      start_point,
      box_size,
      back_color,
      reset_color,
      text_color,
      toast_text,
    };
    wio_toast
  }
  pub fn start(&mut self, 
               toast_text: String<U256>)
  {
    self.toast_text = toast_text;
    self.limit_count = self.limit;
  }

  pub fn count_down(&mut self) 
    -> &mut WioToast
  {
    // Drawableでないとsdl2出力出来ないので
    // count_downは別出しにするしかない。
    if self.limit_count > 0 {
        self.limit_count -= 1;
    }
    self
  }

  fn draw_clear<D>(
    &self, target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>
  {
    Rectangle::new(
       self.start_point,
       self.box_size,
    )
    .into_styled(
      PrimitiveStyle::with_fill(
        self.reset_color)
    )
    .draw(target)
  }

  fn draw_toast<D>(
    &self, target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>
  {
    if self.limit_count > self.limit - 3 {
      Rectangle::new(
        self.start_point,
        self.box_size,
      )
      .into_styled(
        PrimitiveStyle::with_fill(
          self.back_color)
      )
      .draw(target)?;

      Text::new(
        self.toast_text.as_str(),
        Point::new(
          self.start_point.x,
          self.start_point.y +
          self.box_size.height as i32 - 2i32
        ),
        MonoTextStyle::new(
          &FONT_9X15_BOLD,
          self.text_color
        )
      )
      .draw(target).ok().unwrap();
    }
    Ok(())
  }
}

impl Drawable for WioToast
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    match self.limit_count {
      0 => Ok(()),
      1 => {
             self.draw_clear(display)?;
             Ok(())
           },
      _ => {
             self.draw_toast(display)?;
             Ok(())
           }
    }
  }
}


