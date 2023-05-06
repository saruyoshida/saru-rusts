#![no_std]

use nalgebra::{SMatrix, clamp};
use oorandom::Rand32;
use heapless::Vec;
use embedded_graphics::{
  pixelcolor::Rgb565,
  primitives::{PrimitiveStyle, Rectangle},
  prelude::*,
};

const X     : usize = 10;
const Y     : usize = 10;
const X3    : usize = X+3;
const X2    : usize = X+2;
const Y3    : usize = Y+3;
const Y2    : usize = Y+2;

const SP    : usize = 1024;
const SPXY  : usize = 2;
const DELTAT: f32 = 0.5;
const OMEGA : f32 = 1.8;

const LINE_COLOR: Rgb565 = Rgb565::BLUE;
const BG_COLOR  : Rgb565 = Rgb565::WHITE;
// const DRAW_WITH: u32 = 1;
// const CLAR_WITH: u32 = 1;
const SCALE   : f32 = 20.0;
const PRTSIZE : u32 = 2;
const OFFSET  : i32 = 40;

type MatrixX3xY2  = SMatrix<f32, X3, Y2>;
type MatrixX2xY3  = SMatrix<f32, X2, Y3>;
type MatrixX2xY2  = SMatrix<f32, X2, Y2>;
type Matrix1024x2 = SMatrix<f32, SP, SPXY>;

pub struct CfdNsSor {
  vx : MatrixX3xY2,
  vx2: MatrixX3xY2,
  vy : MatrixX2xY3,
  vy2: MatrixX2xY3,
  p  : MatrixX2xY2,
  s  : MatrixX2xY2,
  prt: Matrix1024x2,
  d_cur: Vec<Point, SP>,
  d_pre: Vec<Point, SP>,
  d_onoff: bool,
}

impl CfdNsSor {
  pub fn new() -> Self {
    let vx  = MatrixX3xY2::zeros();
    let vx2 = MatrixX3xY2::zeros();
    let vy  = MatrixX2xY3::zeros();
    let vy2 = MatrixX2xY3::zeros();
    let p   = MatrixX2xY2::zeros();
    let s   = MatrixX2xY2::zeros();
    // 浮遊物の初期位置をランダムに設定
    let mut prt = Matrix1024x2::zeros();
    let mut rng = Rand32::new(4);
    for i in 0..SP
    {
      prt[(i, 0)]  = rng.rand_float() *
                     X as f32 + 1.0;
      prt[(i, 1)]  = rng.rand_float() *
                     Y as f32 + 1.0;
    }

    let d_cur = Vec::new();
    let d_pre = Vec::new();

    CfdNsSor {
      vx, vx2, vy, vy2, p, s, prt, 
      d_cur, d_pre, d_onoff: true
    }
  }

  pub fn update(&mut self) 
  {
    // 描画領域の入替え
    self.swap_drawpoint();
    // 移流
    self.advection();
    // 外力:適当に速度固定
    self.vx[(4, 6)] = 0.8;
    // ダイバージェンス計算
    self.divergence()
    // 圧力計算
    .poisson()
    // 修正
    .rhs()
    // ここで場の速度、圧力が次のステップ
    // に更新されたことになるので
    // 浮遊物の位置を更新。
    .flowparticles();
  }

  fn advection(&mut self) 
  {
    let vx  = &self.vx;
    let vy  = &self.vy;
    let vx2 = &mut self.vx2;
    let vy2 = &mut self.vy2;

    for i in 1..=X { for j in 1..=Y
    {
    // x方向の移流
      // 壁以外の速度を更新
      if i > 1 
      {
        let u = vx[(i, j)];
        let v = 
        (
          vy[(i-1,j  )] + vy[(i  ,j  )] + 
          vy[(i-1,j+1)] + vy[(i  ,j+1)]
        ) 
        / 4.0;

        let ipp:usize 
            = if u < 0.0 {1} else {0};
        let jpp:usize 
            = if v < 0.0 {1} else {0};

        vx2[(i, j)] = 
          vx[(i, j)] 
          - 
          u * (
                vx[(i  +ipp ,j      )]  - 
                vx[(i-1+ipp ,j      )]
              ) * DELTAT
          - 
          v * (
                vx[(i       ,j  +jpp)] -
                vx[(i       ,j-1+jpp)]
              ) * DELTAT;
      }
      
    // y方向の移流
      // 壁以外の速度を更新
      if j > 1 
      {
        let u = 
        (
          vx[(i, j-1)] + vx[(i+1, j-1)] + 
          vx[(i, j  )] + vx[(i+1, j  )]
        ) / 4.0;

        let v = vy[(i, j)];

        let ipp:usize 
            = if u < 0.0 {1} else {0};
        let jpp:usize
            = if v < 0.0 {1} else {0};

        vy2[(i, j)] = 
          vy[(i, j)] 
          -
          u * (
                vy[(i  +ipp ,j      )]  - 
                vy[(i-1+ipp ,j      )]
              ) * DELTAT
          - 
          v * (
                vy[(i       ,j  +jpp)] -
                vy[(i       ,j-1+jpp)]
              ) * DELTAT;
      }
    }}

    core::mem::swap(
      &mut self.vx,
      &mut self.vx2
    );
    core::mem::swap(
      &mut self.vy,
      &mut self.vy2
    );
  }

  fn divergence(&mut self) -> &mut CfdNsSor
  {
    let vx = &self.vx;
    let vy = &self.vy;
    let s  = &mut self.s;

    for i in 1..=X { for j in 1..=Y
    {
      s[(i, j)]=
      ( 
         -vx[(i  , j)] - vy[(i, j  )] +
          vx[(i+1, j)] + vy[(i, j+1)] 
      )
      / DELTAT;
    }}
    self
  }

  fn poisson(&mut self) -> &mut CfdNsSor
  {
    let p = &mut self.p;
    let s = &self.s;

    for _ in 0..=30 { for i in 1..=X  {
    for j in 1..=Y
    {
      // 左の壁
      if i == 1 { p[(i-1, j  )] = p[(i, j)]; }
      // 右の壁
      if i == X { p[(i+1, j  )] = p[(i, j)]; }
      // 上の壁
      if j == 1 { p[(i  , j-1)] = p[(i, j)]; }
      // 下の壁
      if j == Y { p[(i  , j+1)] = p[(i, j)]; }

      p[(i, j)] = 
        (1.0 - OMEGA) * p[(i, j)] 
        + 
        OMEGA / 4.0
          * 
          (
            p[(i-1, j  )] + p[(i+1, j  )] +
            p[(i  , j-1)] + p[(i  , j+1)] 
            -
            s[(i, j)]
          );
    }}}
    self
  }

  fn rhs(&mut self) -> &mut CfdNsSor
  {
    let vx = &mut self.vx;
    let vy = &mut self.vy;
    let p  = &self.p;

    for i in 1..=X { for j in 1..=Y
    {
      vx[(i, j)] -=
        (p[(i, j)] - p[(i-1, j  )]) * DELTAT;
      vy[(i, j)] -= 
        (p[(i, j)] - p[(i  , j-1)]) * DELTAT;
    }}
    self
  }

  fn flowparticles(&mut self) -> &mut CfdNsSor
  {
    let vx  = &self.vx;
    let vy  = &self.vy;
    let prt = &mut self.prt;

    for i in 0..SP
    {
      let xx = clamp
               (
                 prt[(i, 0)], 
                 0.0, 
                 (X+3-2) as f32
               );

      let yy = clamp
               (
                 prt[(i, 1)],
                 0.0,
                 (Y+3-2) as f32
               );

      let ixx =  xx      as usize;
      let iyy = (yy-0.5) as usize;

      let sxx =  xx      - ixx as f32;
      let syy = (yy-0.5) - iyy as f32;
      let spdx = 
      (
        (
          (1.0 - sxx) * vx[(ixx  , iyy  )] + 
                 sxx  * vx[(ixx+1, iyy  )] 
        )  
        *
        (1.0 - syy) +  
        (
          (1.0 - sxx) * vx[(ixx  , iyy+1)] + 
                 sxx  * vx[(ixx+1, iyy+1)] 
        )  
        * 
        syy
      ) * DELTAT;

      let ixx = (xx-0.5) as usize;
      let iyy =  yy      as usize;

      let sxx = (xx-0.5) - ixx as f32;
      let syy =  yy      - iyy as f32;
      let spdy = 
      (
        (
          (1.0 - sxx) * vy[(ixx  , iyy  )] + 
                 sxx  * vy[(ixx+1, iyy  )]
        ) 
        * 
        (1.0 - syy) + 
        (
          (1.0 - sxx) * vy[(ixx  , iyy+1)] + 
                 sxx  * vy[(ixx+1, iyy+1)]
        ) 
        *
        syy
      ) * DELTAT;

      prt[(i, 0)] += spdx;
      prt[(i, 1)] += spdy;

      self.d_cur.push(
        Point::new(
          (prt[(i, 0)] * SCALE) as i32
                       + OFFSET,
          (prt[(i, 1)] * SCALE) as i32
        )
      ).unwrap();
    }
    self
  }
}

impl CfdNsSor
{
  fn swap_drawpoint(&mut self)
  {
    core::mem::swap(
      &mut self.d_cur,
      &mut self.d_pre
    );
    self.d_cur.clear();
  }

  pub fn draw_on(&mut self) -> &mut CfdNsSor
  {
    self.d_onoff = true;
    self
  }

  pub fn draw_off(&mut self) -> &mut CfdNsSor
  {
    self.d_onoff = false;
    self
  }
}

impl Drawable for CfdNsSor
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) -> 
    Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    let mut p_color      = LINE_COLOR;
    let mut p_points     = &self.d_cur;
//  let mut stroke_width = DRAW_WITH;
           
    if !(self.d_onoff) {
      p_color      = BG_COLOR;
      p_points     = &self.d_pre;
//    stroke_width = CLAR_WITH;
    }

    for xy in p_points.iter()
    {
      Rectangle::new(
        *xy, 
        Size::new(PRTSIZE,PRTSIZE)           
      )
      .into_styled(
         PrimitiveStyle::with_fill(
           p_color
         )
      )
      .draw(display)?;
    }
    Ok(())
  }
}            
    


