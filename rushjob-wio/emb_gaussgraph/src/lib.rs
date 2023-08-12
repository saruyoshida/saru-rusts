#![no_std]

use micromath::F32Ext;
use emb_bargraph::*;
use emb_linegraph::*;
use core::f32::consts::PI;

// ガウス分布
#[derive(Clone)]
pub struct Gaussian {
  pub mean : f32,        // 平均
  pub var  : f32,        // 分散
}
// ガウス分布グラフ
#[derive(Clone)]
pub struct EmbGaussgraph {
  line_graph    : EmbLinegraph, // 線グラフ
  x_scale_range : Range<i32>,   // X目盛レンジ
  correct_rate  : (f32, f32),   // 補正率
  data          : Gaussian,     // 平均、分散
}
// new
impl EmbGaussgraph {
  pub fn new(
    graph : &EmbBargraph,       // 棒グラフ
  ) -> Self
  {
    EmbGaussgraph {
      line_graph   : EmbLinegraph::new(graph),
      x_scale_range: graph.x_scale_range(),
      correct_rate : graph.correct_rate(),
      data         : Gaussian{
                       mean : 0.0,
                       var  : 1.0
                     },
    }
  }
}
// 図形モード、データセット
impl EmbGaussgraph {
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
    mean : f32,
    var  : f32,
  ) -> &mut Self
  {
    self.data.mean = mean;
    self.data.var  = var;
    self
  }
}
// 描画
impl Drawable for EmbGaussgraph
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    let range   = self.x_scale_range.clone();
    let mut elg = self.line_graph.clone();
    for i in range {
      let x = i as f32 / self.correct_rate.0;
      elg.set_data(x, self.gaussian(x))
          .draw(display)?;
    }
    Ok(())
  }
}
// ガウス分布計算
impl EmbGaussgraph {
  fn gaussian(
    &self,
    x : f32,
  ) -> f32
  {
//  ((2*math.pi*var)**-.5) * 
//  np.exp(
//          (-0.5*(np.asarray(x)-mean)**2.) /
//          var
//        )
    1.0 /
    (
      self.data.var.sqrt() * 
      (2.0 * PI).sqrt() 
    ) *
    (
      (x - self.data.mean).powf(2.0) /
      (-2.0 * self.data.var)
    ).exp()
  }
}
// その他セッター
impl EmbGaussgraph {
  pub fn set_shape_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.line_graph.set_shape_color(c);
    self
  }

  pub fn set_shape_width(&mut self, c: u32)
    -> &mut Self {
    self.line_graph.set_shape_width(c);
    self
  }

  pub fn set_dot_interval(&mut self, c: usize)
    -> &mut Self {
    self.line_graph.set_dot_interval(c);
    self
  }
}
