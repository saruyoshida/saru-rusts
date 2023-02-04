#![no_std]

use nalgebra::SMatrix;
use embedded_graphics::{
  pixelcolor::Rgb565,
  primitives::{PrimitiveStyle, Rectangle},
  prelude::*,
};

const L:  usize = 60;
const DT: f32 = 0.2;
const F:  f32 = 0.04;
const K:  f32 = 0.06075;
const DU: f32 = 0.1;
const DV: f32 = 0.05;
const SCALE: i32 = 4;
const OFFSET: i32 = 40;

type MatrixLxL = SMatrix<f32, L, L>;

pub struct WioReactDiff {
  u:  MatrixLxL,
  u2: MatrixLxL,
  v:  MatrixLxL,
  v2: MatrixLxL,
}

impl WioReactDiff {
  pub fn new() -> Self 
  {
    let     u  = MatrixLxL::zeros();
    let mut u2 = MatrixLxL::zeros();
    let     v  = MatrixLxL::zeros();
    let mut v2 = MatrixLxL::zeros();

    let h = L / 2;

    // u[h-6:h+6, h-6:h+6] = 0.9
    // u2.fixed_slice_mut::<12, 12>(h-6, h-6)
    //   .add_scalar_mut(0.9);
    u2.slice_range_mut(h-6..h+6, h-6..h+6)
      .fill(0.9);

    // v[h-3:h+3, h-3:h+3] = 0.7
    // v2.fixed_slice_mut::<6, 6>(h-3, h-3)
    //   .add_scalar_mut(0.7);
    v2.slice_range_mut(h-3..h+3, h-3..h+3)
      .fill(0.7);

    let wio_reactdiff = WioReactDiff {
      u, u2, v, v2,
    };
    wio_reactdiff
  }

  pub fn update(&mut self) 
    -> &mut WioReactDiff
  {
    let dt = DT;
    let k  = K;
    #[allow(non_snake_case)]
    let Du = DU;
    #[allow(non_snake_case)]
    let Dv = DV;
   
    core::mem::swap(
      &mut self.u,
      &mut self.u2,
    );
    core::mem::swap(
      &mut self.v,
      &mut self.v2,
    );
    
    let mut lu = MatrixLxL::zeros();
    let mut lv = MatrixLxL::zeros();

    for ix in 1..L-1 {
      for iy in 1..L-1 {
        lu[(ix, iy)] 
          = self.laplac(ix,iy,&self.u);
        lv[(ix, iy)] 
          = self.laplac(ix,iy,&self.v);
      }
    }

//  v*v*u
    let vvu 
      = (&self.v).component_mul(&self.v)
                 .component_mul(&self.u);

//  u2 = u + (Dulu - vvu + F(1.0 - u))dt
    self.u2 = self.u +
      (
        (
          ((Du * lu) - vvu) +
          (
            F *
            ((self.u * -1f32)
             .add_scalar(1.0))
          )
        ) * dt
      );
//  v2 = v + (Dvlv + vvu - (F + k)v)dt
    self.v2 = self.v + 
      (
        (
          ((Dv * lv) + vvu) - 
          ((F + k) * self.v)
        ) * dt
      );

    self
  }

  fn laplac(
    &self,
    ix: usize, 
    iy: usize,
    s: &MatrixLxL
  ) -> f32
  {
    let ts: f32 = s[(ix-1, iy)]
                + s[(ix+1, iy)]
                + s[(ix  , iy-1)]
                + s[(ix  , iy+1)]
                - 4.0 * s[(ix, iy)];
    ts
  }
}

impl Drawable for WioReactDiff
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) -> 
    Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    for x in 0..L {
      for y in 0..L {
        let u_pix: u8 = 
          (Rgb565::MAX_G as f32 * 
           self.u[(x, y)]) 
          as u8;
        let u2_pix: u8 = 
          (Rgb565::MAX_G as f32 * 
           self.u2[(x, y)]) 
          as u8;

        if u_pix == u2_pix { continue }

        Rectangle::new(
          Point::new(
            x as i32 * SCALE + OFFSET,
            y as i32 * SCALE
          ), 
          Size::new(SCALE as u32,
                    SCALE as u32),
        )
        .into_styled(
          PrimitiveStyle::with_fill(
            Rgb565::new(0, u2_pix, 0)
          )
        )
        .draw(display)?;

//        Pixel(
//          Point::new(x as i32, y as i32), 
//          Rgb565::new(0, u2_pix, 0)
//        )
//        .draw(display)?;
      }
    }
    Ok(())
  }
}            
    


