#![no_std]

use core::f32::consts::PI;

use embedded_graphics::{
  pixelcolor::Rgb565,
  prelude::*,
  primitives::{Polyline,PrimitiveStyle},
};

use micromath::F32Ext;

use heapless::consts::*;
use heapless::Vec;

const LINE_COLOR: Rgb565 = Rgb565::WHITE;
const BG_COLOR: Rgb565 = Rgb565::BLACK;
const DRAW_WITH: u32 = 1;
const CLAR_WITH: u32 = 1;

pub struct WioElliptClock {
  ct: i32,            // 進度  :0〜359
  ctk: i32,           // 進度係数
  x0y0: (i32, i32),   // 中心点
  x1y1: (i32, i32),   // 終点
  a: f32,             // 長軸率:0.1〜1.0
  b: f32,             // 短軸率:0.1〜1.0
  th: i32,            // 基準角度
  dp: i32,            // 解像度:0,3,4,〜
  r: f32,             // 半径
  ce: i32,            // 0:楕円、1真円
  x0y0_prev: (i32, i32),
  x1y1_prev: (i32, i32),
  poly_cur:  Vec<Point, U361>,
  poly_prev: Vec<Point, U361>,
  draw_onoff: bool,
}

impl WioElliptClock {
  pub fn new(
    ctk: i32,           // 進度係数
    r: f32,             // 半径
  ) -> Self 
  {
    let wio_elliptclock = WioElliptClock {
      ct:0i32,            // 進度  :0〜359
      ctk,                // 進度係数
      x0y0: (0i32, 0i32), // 中心点
      x1y1: (0i32, 0i32), // 終点
      a: 1.0f32,          // 長軸率:0.1〜1.0
      b: 1.0f32,          // 短軸率:0.1〜1.0
      th: 0i32,           // 基準角度
      dp: 360i32,         // 解像度:1〜360
      r,                  // 半径
      ce: 0i32,           // 真円楕円区分
      x0y0_prev: (0, 0),
      x1y1_prev: (0, 0),
      poly_cur: Vec::new(),
      poly_prev: Vec::new(),
      draw_onoff: true,
    };
    wio_elliptclock
  }

  pub fn reset(
    &mut self,
    a: f32,        // 長軸率:0.1〜1.0
    b: f32,        // 短軸率:0.1〜1.0
    dp: i32,       // 解像度:0,3,4,〜
    ce: i32,       // 真円楕円区分
  )
  {
    self.a = a;
    self.b = b;
    self.dp = dp;
    self.ce = ce;
  }

  pub fn update(
    &mut self,
    ct: i32,            // 進度
    x0y0: (i32, i32),   // 中心点
    th: i32,            // 基準角度
  ) -> ((i32, i32), i32)
  {
    self.x0y0_prev = self.x0y0;
    self.x1y1_prev = self.x1y1;

    core::mem::swap(
      &mut self.poly_cur,
      &mut self.poly_prev,
    );
    
    self.poly_cur.clear();

    self.ct = ct;
    self.x0y0 = x0y0;
    self.th = th;

    let ctr = self.ct * self.ctk;

    self.x1y1 = self.laps_apex(
      self.a,
      self.b,
      self.r,
      ctr,
    );

    let r = 
      ( 
        (
          (self.x1y1.0 - x0y0.0 ).pow(2) +
          (self.x1y1.1 - x0y0.1 ).pow(2)
        ) as f32
      ).sqrt();

    self.poly_cur.push(
       Point::new(x0y0.0, x0y0.1)
    ).unwrap();

    let sk = 360 / self.dp;
    let mut x2y2 = (0i32, 0i32);

    for ct in (ctr..(360 + ctr))
              .step_by(sk as usize)
    {
      let x1y1 = if self.ce == 0 
      {
        self.laps_apex(
          self.a,
          self.b,
          self.r,
          ct,
        )
      } else {
        self.laps_apex(
          1.0,
          1.0,
          r,
          ct,
        )
      };

      if x1y1 != x2y2 {
        self.poly_cur.push(
          Point::new(x1y1.0,x1y1.1)
        ).unwrap();
    
        x2y2 = x1y1;
      }
    }
    self.poly_cur.push(
        self.poly_cur[1]
    ).unwrap();

    (
     (self.poly_cur[1].x,
      self.poly_cur[1].y),
     ctr
    )
  }

  fn laps_apex(
    &self,
    a: f32,
    b: f32,
    r: f32,
    ct: i32,
  ) -> (i32, i32)
  {
    let th = self.th as f32 / 180f32 * PI;
    let w  = 
      if ct == 0 {
        0f32
      } else {
        ct as f32 / 180f32 * PI
      };

    let x1 = a * r * w.cos() * th.cos()
           - b * r * w.sin() * th.sin()
           + self.x0y0.0 as f32;

    let y1 = a * r * w.cos() * th.sin()
           + b * r * w.sin() * th.cos()
           + self.x0y0.1 as f32;

    (x1 as i32, y1 as i32)
  }

  pub fn draw_on(&mut self) 
  -> &mut WioElliptClock
  {
    self.draw_onoff = true;
    self
  }

  pub fn draw_off(&mut self) 
  -> &mut WioElliptClock
  {
    self.draw_onoff = false;
    self
  }

  fn draw_laps<D>(
    &self, 
    target: &mut D,
    draw_color: Rgb565,
    stroke_width: u32,
    points: &Vec<Point, U361>,
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
 
impl Drawable for WioElliptClock
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
    
