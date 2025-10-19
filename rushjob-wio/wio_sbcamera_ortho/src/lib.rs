#![no_std]
extern crate nalgebra as na;
use na::*;
use core::f32::consts::PI;

type T = f32;
// 球面束縛カメラ:正射投影
pub struct WioSBCameraOrtho {
  eye       : Point3<T>,        // カメラ位置
  target    : Point3<T>,        // 注視点
  model     : Isometry3<T>,     // モデル位置
  projection: Orthographic3<T>, // 正射投影
  screen    : Matrix4<T>,       // スクリーン
}
impl WioSBCameraOrtho {
  pub fn new (
    dspw: f32, // 画面幅
    dsph: f32, // 画面高さ
    dspd: f32, // 画面奥行き
  ) -> Self {
    WioSBCameraOrtho {
      eye   : Point3::new(0., 0., -dspd / 2.),
      target: Point3::new(0., 0., 0.),
      model : Isometry3::new(
        Vector3::new(
          // モデルの中心が原点に来るよう移動
          dspw / -2., dsph /  2., dspd / 2.,
        ),
        Vector3::x() * PI,
      ),
      projection: Orthographic3::new(
        dspw*-0.7,  // left
        dspw* 0.7,  // right 
        dsph*-0.7,  // bottom
        dsph* 0.7,  // top 
        dspd*-0.7,  // znear
        dspd* 0.7,  // zfar
      ),
      screen: Matrix4::new(
        dspw / 2., 0.        , 0., dspw / 2.,
        0.       , -dsph / 2., 0., dsph / 2.,
        0.       , 0.        , 1., 0.       ,
        0.       , 0.        , 0., 1.       ,
      ),
    }
  }
}
impl WioSBCameraOrtho {
  pub fn convertf32(&self, p: (T, T, T)) 
  -> (T, T) {
    let view = Isometry3::look_at_rh(
      &self.eye,
      &self.target, 
      &Vector3::y()
    );
    let mvp = 
      self.projection.into_inner() * 
      (view * self.model).to_homogeneous()
    ;
    let vertex = 
      mvp * Vector4::new(p.0, p.1, p.2, 1.0)
    ;  
    self.viewportf32(&vertex)
  }
  fn viewportf32(&self, vertex: &Vector4<T>) 
  -> (f32, f32) {
    let w = if vertex.w <= 0.0 {
              0.001
            } else {
              vertex.w
            }
    ;
    let v = self.screen * vertex / w;
    (v.x, v.y)
  }
}
// セッター
impl WioSBCameraOrtho {
  pub fn set_eye(&mut self, t: (T, T, T))
  -> &mut Self {
    let p = &mut self.eye;
    (p.x, p.y, p.z) = (t.0, t.1, t.2);
    self
  }
  pub fn set_model(
    &mut self, 
    t: (T, T, T),
    r: (T, T, T)
  ) -> &mut Self {
    self.model = Isometry3::new(
      Vector3::new(t.0, t.1, t.2),
      Vector3::new(r.0, r.1, r.2),
    );
    self
  }
  pub fn set_left_and_right(
    &mut self,
    left:  T,
    right: T,
  ) -> &mut Self {
    self.projection.set_left_and_right(
      left, right
    );
    self
  }
  pub fn set_bottom_and_top(
    &mut self,
    bottom: T,
    top   : T,
  ) -> &mut Self {
    self.projection.set_bottom_and_top(
      bottom, top
    );
    self
  }
  pub fn set_znear_and_zfar(
    &mut self,
    znear: T,
    zfar : T,
  ) -> &mut Self {
    self.projection.set_znear_and_zfar(
      znear, zfar
    );
    self
  }
  pub fn set_target(
    &mut self, 
    x: T,
    y: T,
    z: T,
  ) -> &mut Self {
    self.target.x = x;
    self.target.y = y;
    self.target.z = z;
    self
  }
}


