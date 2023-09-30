#![no_std]

extern crate nalgebra as na;
pub use na::{Matrix2, Vector2};
use micromath::F32Ext;
use heapless::Vec;
use emb_bargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;
use core::f32::consts::PI;

// 分散楕円グラフ
#[derive(Clone)]
pub struct EmbCovargraph {
  line_graph  : EmbLinegraph, // 線グラフ
  shape_graph : EmbShapegraph,// 図形グラフ
  x           : f32,          // 中心x
  y           : f32,          // 中心y
  th          : f32,          // 楕円角度
  a           : f32,          // 長軸率
  b           : f32,          // 短軸率
  r           : f32,          // 半径
  std         : Vec<f32, 10>, // 標準偏差
  step        : usize,        
}
// new
impl EmbCovargraph {
  pub fn new(
    graph : &EmbBargraph,    // 棒グラフ
  ) -> Self
  {
    let mut sg = EmbShapegraph::new(graph);
    sg.mode_fillcircle();
    
    EmbCovargraph {
      line_graph  : EmbLinegraph::new(graph),
      shape_graph : sg,
      x           : 0.0,
      y           : 0.0,
      th          : 0.0,
      a           : 1.0,
      b           : 1.0,
      r           : 1.0,
      std         : Vec::from_slice(&[1.0])
                        .unwrap(),
      step        : 3,
    }
  }
}
// 図形モード、データセット
impl EmbCovargraph {
  // 実線
  pub fn mode_realline(
    &mut self
  ) -> &mut Self
  {
    self.line_graph.mode_realline();
    self
  }
  // 点線
  pub fn mode_dotline(
    &mut self
  ) -> &mut Self
  {
    self.line_graph.mode_dotline();
    self
  }
  //データセット
  pub fn set_data(
    &mut self,
    m: &[f32],
    c: &[f32],
  ) -> &mut Self
  {
    let mean = Vector2::<f32>
                 ::from_column_slice(m);
    let covar = Matrix2::<f32>
                 ::from_column_slice(c);
    // 中心点
    (self.x, self.y) = (mean.x, mean.y);
    self.shape_graph.set_data(
      self.x, self.y
    );
    // 特異値分解
    let svd = covar.svd(true, true);
    let u = svd.u.unwrap();

    self.th = u.m21.atan2(u.m11);  // 角度
    let s = svd.singular_values;   
    self.r = s.x.sqrt();           // 半径
    self.b = s.y.sqrt() / self.r;  // 短軸率

    self
  }
}
// 描画
impl Drawable for EmbCovargraph
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    let mut elg = self.line_graph.clone();
    // 中心点描画
    self.shape_graph.draw(display)?;
    // 楕円始点
    let (mut x0, mut y0) = (0.0, 0.0);
    // 標準偏差毎
    for s in self.std.iter() {
      // 楕円描画
      for i in (0..359).step_by(self.step) {
        // 楕円頂点
        let (x, y) = self.eli_vertex(i, *s);
        // 楕円始点取得
        if i == 0 {(x0, y0) = (x, y);}
        // 描画
        elg.set_data(x, y).draw(display)?;
      }
      // 楕円始点に戻る
      elg.set_data(x0, y0).draw(display)?;
      elg.reset_data();
    }
    Ok(())
  }
}
// 楕円頂点
impl EmbCovargraph {
  fn eli_vertex(&self, ct: usize, s: f32)
    -> (f32, f32)
  {
    let (x, y) = (self.x, self.y);
    let th     = self.th;
    let (a, b) = (self.a, self.b);
    let r      = self.r * s;

    let w  =  ct as f32 / 180.0 * PI;

    let x1 = a * r * w.cos() * th.cos()
           - b * r * w.sin() * th.sin()
           + x;

    let y1 = a * r * w.cos() * th.sin()
           + b * r * w.sin() * th.cos()
           + y;

    (x1,  y1)
  }
}
// その他セッター
impl EmbCovargraph {
  // 描画ステップ
  pub fn set_step(&mut self, step: usize)
    -> &mut Self
  {
    self.step = step;
    self
  }
  // 標準偏差
  pub fn set_std(&mut self, std: &[f32])
    -> &mut Self
  {
    self.std = Vec::from_slice(std).unwrap();
    self
  }

  pub fn set_shape_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.line_graph.set_shape_color(c);
    self.shape_graph.set_shape_color(c);
    self
  }

  pub fn set_shape_width(&mut self, c: u32)
    -> &mut Self {
    self.line_graph.set_shape_width(c);
    self.shape_graph.set_shape_width(c);
    self
  }

  pub fn set_dot_interval(&mut self, c: usize)
    -> &mut Self {
    self.line_graph.set_dot_interval(c);
    self
  }
}

