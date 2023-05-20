#![no_std]

use embedded_graphics::{
  pixelcolor::Rgb565,
  prelude::*,
  primitives::{Polyline,PrimitiveStyle},
};
use heapless::consts::*;
use heapless::Vec;


const LINE_COLOR: Rgb565 = Rgb565::GREEN;
const BG_COLOR: Rgb565 = Rgb565::BLACK;
const DRAW_WITH: u32 = 1;
const CLAR_WITH: u32 = 1;

pub struct WioPolyWave {
  x0: i32,   // 中心点
  x1: i32,   // 終点
  poly_cur:  Vec<Point, U321>,
  poly_prev: Vec<Point, U321>,
  draw_onoff: bool,
// 追加 
  poly_cur_draw:  Vec<Point, U321>,
  poly_prev_draw: Vec<Point, U321>,
//
}

impl WioPolyWave {
  pub fn new(
    x0: i32,
    x1: i32,
  ) -> Self 
  {
    let wio_polywave = WioPolyWave {
      x0,
      x1,
      poly_cur: Vec::new(),
      poly_prev: Vec::new(),
      draw_onoff: true,
// 追加
      poly_cur_draw: Vec::new(),
      poly_prev_draw: Vec::new(),
//
    };
    wio_polywave
  }

  pub fn update(
    &mut self,
    x0y0: (i32, i32), 
  )
  {
    core::mem::swap(
      &mut self.poly_cur,
      &mut self.poly_prev,
    );
    
    self.poly_cur.clear();

    self.poly_cur.push(
       Point::new(x0y0.0, x0y0.1)
    ).unwrap();

    self.poly_cur.push(
       Point::new(self.x0, x0y0.1)
    ).unwrap();

    let mut x2 = self.x0;

    for points in self.poly_prev.iter()
                    .skip(1)
    {
      x2 += 1;
      if x2 < self.x1 {
        self.poly_cur.push(
          Point::new(x2, points.y)
        ).unwrap();
      }
    }
  }

  pub fn draw_on(&mut self) 
  -> &mut WioPolyWave
  {
    self.draw_onoff = true;
    self
  }

  pub fn draw_off(&mut self) 
  -> &mut WioPolyWave
  {
    self.draw_onoff = false;
    self
  }

  fn draw_laps<D>(
    &self, 
    target: &mut D,
    draw_color: Rgb565,
    stroke_width: u32,
    points: &Vec<Point, U321>,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let line_style = 
      PrimitiveStyle::with_stroke(
        draw_color, stroke_width
      );
      
    Polyline::new(points)
      .into_styled(line_style)
      .draw(target)
  }
}   
 
impl Drawable for WioPolyWave
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) -> 
    Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    
    let mut laps_color   = LINE_COLOR;
    let mut poly_points = &self.poly_cur;
    let mut stroke_width = DRAW_WITH;
           
    if !(self.draw_onoff) {
      laps_color   = BG_COLOR;
      poly_points = &self.poly_prev;
      stroke_width = CLAR_WITH;
    }
    
    self.draw_laps(
      display, 
      laps_color,
      stroke_width,
      poly_points,
    )
  }
}            
    
// 追加
impl WioPolyWave {
  pub fn as_mut_points(&mut self) 
    -> &mut [Point] 
  {
    &mut self.poly_cur
  }
}

impl WioPolyWave {
  pub fn swap_start(&mut self) {
    self.poly_cur_draw = 
      self.poly_cur.clone();

    core::mem::swap(
      &mut self.poly_prev,
      &mut self.poly_prev_draw,
    );
  }

  pub fn swap_end(&mut self) {
    core::mem::swap(
      &mut self.poly_cur,
      &mut self.poly_cur_draw,
    );
    core::mem::swap(
      &mut self.poly_prev,
      &mut self.poly_prev_draw,
    );
    core::mem::swap(
      &mut self.poly_cur_draw,
      &mut self.poly_prev_draw,
    );
  }
}
//

