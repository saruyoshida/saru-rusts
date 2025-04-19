#![no_std]

extern crate nalgebra as na;
use na::*;
use embedded_graphics::geometry::Point;
use core::f32::consts::PI;

// spherical-bound camera
pub struct WioSBCamera {
  eye       : Point3<f32>,  // カメラ位置
  target    : Point3<f32>,  // 注視点
  model     : Isometry3<f32>,
  projection: Perspective3<f32>,
  screen    : Matrix4<f32>,
}

impl WioSBCamera {
  pub fn build_default (
    dspw: f32,
    dsph: f32,
  ) -> Self 
  {
    WioSBCamera {
      eye   : Point3::new(
        0.0, 0.0, dspw / 2.0
      ),
      target: Point3::new(0.0, 0.0, 0.0),
      model : Isometry3::new(
        Vector3::new(
          dspw  /  -2.0,
          dsph  /   2.0,
          0.0,
        ),
        Vector3::x() * PI,
      ),
      projection: Perspective3::new(
        dspw / dsph,
        3.14 / 4.0, 
        dspw / 2.0,
        dspw * 2.0,
      ),
      screen: Matrix4::new(
        dspw / 2., 0.        , 0., dspw / 2.,
        0.       , -dsph / 2., 0., dsph / 2.,
        0.       , 0.        , 1., 0.       ,
        0.       , 0.        , 0., 1.       ,
      ),
    }
  }
  
  pub fn convert(
    &self,
    points: &mut [Point],
  ) 
  {
    let view = Isometry3::look_at_rh(
      &self.eye,
      &self.target, 
      &Vector3::y()
    );

    let mvp = 
      self.projection.into_inner() * 
      (view * self.model).to_homogeneous();

    for p in points {
      let vertex = 
        mvp *
        Vector4::new(
          p.x as f32,
          p.y as f32,
          0.0,
          1.0,
        );

      (p.x, p.y) = self.viewport(&vertex);
    }
  }

  fn viewport(
    &self,
    vertex: &Vector4<f32>
  ) -> (i32, i32)
  {
    let w = if vertex.w <= 0.0 {
              0.001
            } else {
              vertex.w
            };

    let v = self.screen * vertex / w;

    (v.x as i32, v.y as i32)
  }
}
// 20250402 add start
impl WioSBCamera {
  pub fn convertf32(
    &self,
    p: (f32, f32, f32),
  ) -> (f32, f32) {
    let view = Isometry3::look_at_rh(
      &self.eye,
      &self.target, 
      &Vector3::y()
    );
    let mvp = 
      self.projection.into_inner() * 
      (view * self.model).to_homogeneous();
    let vertex = 
      mvp * Vector4::new(p.0, p.1, p.2, 1.0)
    ;  
    self.viewportf32(&vertex)
  }

  fn viewportf32(
    &self,
    vertex: &Vector4<f32>
  ) -> (f32, f32) {
    let w = if vertex.w <= 0.0 {
              0.001
            } else {
              vertex.w
            };
    let v = self.screen * vertex / w;
    (v.x, v.y)
  }
}
// 20250402 add end

impl WioSBCamera {
  pub fn set_eye(
    &mut self, 
    t: (f32, f32, f32)
  ) -> &mut Self
  {
    let p = &mut self.eye;
    (p.x, p.y, p.z) = (t.0, t.1, t.2);
    self
  }

  pub fn set_model(
    &mut self, 
    t: (f32, f32, f32),
    r: (f32, f32, f32)
  ) -> &mut Self
  {
    self.model =
      Isometry3::new(
        Vector3::new(t.0, t.1, t.2),
        Vector3::new(r.0, r.1, r.2),
      );
    self
  }

  pub fn set_aspect(&mut self, t:f32) 
    -> &mut Self
  {
    self.projection.set_aspect(t);
    self
  }

  pub fn set_fovy(&mut self, t:f32) 
    -> &mut Self
  {
    self.projection.set_fovy(t);
    self
  }

  pub fn set_znear(&mut self, t:f32) 
    -> &mut Self
  {
    self.projection.set_znear(t);
    self
  }

  pub fn set_zfar(&mut self, t:f32) 
    -> &mut Self
  {
    self.projection.set_zfar(t);
    self
  }
}

