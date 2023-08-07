#![no_std]

use emb_bargraph::*;

pub use embedded_graphics::{
  primitives::{
    Triangle,
    Circle,
  },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ShapeMode {
  RevTriangle,
  Triangle,
  Circle,
  Rectangle,
  FillRevTriangle,
  FillTriangle,
  FillCircle,
  FillRectangle,
}
#[derive(Clone)]
pub struct EmbShapegraph {
  scale_start   : Point,     // 目盛原点
  bar_width     : i32,       // バー幅
  x_scale_start : i32,       // X目盛開始位置
  draw_area     : Rectangle, // draw領域
  correct_shift : (f32, f32),// 位置シフ(x,y)
  correct_fact  : (f32, f32),// 位置係数(x,y)
  shape_mode    : ShapeMode, // 図形モード
  shape_color   : Rgb565,    // 図形色
  shape_width   : u32,       // 図形線幅
  shape_diameter: u32,       // 図形直径
  data          : (f32, f32),// データ
}
// new
impl EmbShapegraph {
  pub fn new(
    graph : &EmbBargraph,        // 棒グラフ
  ) -> Self
  {
    EmbShapegraph {
      scale_start   : graph.scale_start(),
      bar_width     : graph.bar_width(),
      x_scale_start : graph.x_scale_start(),
      draw_area     : graph.draw_area(),
      correct_shift : graph.correct_shift(),
      correct_fact  : graph.correct_fact(),
      shape_mode    : ShapeMode::RevTriangle,
      shape_color   : Rgb565::RED,
      shape_width   : 1,
      shape_diameter: 4,
      data          : (0.0, 0.0),
    }
  }
}
// 図形モード、データセット
impl EmbShapegraph {
  // 逆三角形
  pub fn mode_revtriangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::RevTriangle;
    self
  }
  // 三角形
  pub fn mode_triangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::Triangle;
    self
  }
  // 円
  pub fn mode_circle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::Circle;
    self
  }
  // 四角形
  pub fn mode_rectangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::Rectangle;
    self
  }
  // 逆三角形塗潰し
  pub fn mode_fillrevtriangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = 
      ShapeMode::FillRevTriangle;
    self
  }
  // 三角形塗潰し
  pub fn mode_filltriangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::FillTriangle;
    self
  }
  // 円塗潰し
  pub fn mode_fillcircle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::FillCircle;
    self
  }
  // 四角形塗潰し
  pub fn mode_fillrectangle(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = 
      ShapeMode::FillRectangle;
    self
  }
  //データセット
  pub fn set_data(
    &mut self,
    x : f32,
    y : f32,
  ) -> &mut Self
  {
    self.data = (x, y);
    self
  }
}
// 描画
impl Drawable for EmbShapegraph
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    // サイズ
    let ysize =
      (self.data.1 * self.correct_fact.1 -
       self.correct_shift.1 
       ) as i32;
    // 表示ポイント
    let p = Point::new(
      // X
      self.x_scale_start 
      + 
      (self.data.0 * self.correct_fact.0 -
       self.correct_shift.0
      ) as i32 
      -
      self.bar_width / 2,
      // Y
      self.scale_start.y - ysize,
    );
    // draw領域外チェック
    if !self.draw_area.contains(p) {
      return Ok(());
    }
    // 図形表示
    match self.shape_mode {
      ShapeMode::RevTriangle |
      ShapeMode::FillRevTriangle => {
        self.draw_revtriangle(display, p)?;
      },
      ShapeMode::Triangle |
      ShapeMode::FillTriangle => {
        self.draw_triangle(display, p)?;
      },
      ShapeMode::Circle |
      ShapeMode::FillCircle => {
        self.draw_circle(display, p)?;
      },
      ShapeMode::Rectangle |
      ShapeMode::FillRectangle => {
        self.draw_rectangle(display, p)?;
      },
    };
    Ok(())
  }
}
// 図形表示
impl EmbShapegraph { 
  // 逆三角形
  fn draw_revtriangle<D>(
    &self, 
    target : &mut D,
    p      : Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let loc = self.shape_diameter as i32 / 2 ;
    Triangle::new(
      p, 
      Point::new(
        p.x - loc,
        p.y - loc - loc / 2,
      ),
      Point::new(
        p.x + loc,
        p.y - loc - loc / 2,
      ),
    )
    .into_styled(self.shape_style())
    .draw(target)
  }
  // 三角形
  fn draw_triangle<D>(
    &self, 
    target : &mut D,
    p      : Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let loc = self.shape_diameter as i32 / 2 ;
    Triangle::new(
      p, 
      Point::new(
        p.x - loc,
        p.y + loc + loc / 2,
      ),
      Point::new(
        p.x + loc,
        p.y + loc + loc / 2,
      ),
    )
    .into_styled(self.shape_style())
    .draw(target)
  }
  // 円
  fn draw_circle<D>(
    &self, 
    target : &mut D,
    p      : Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let loc = self.shape_diameter as i32 / 2 ;
    Circle::new(
      Point::new(
        p.x - loc,
        p.y - loc,
      ), 
      self.shape_diameter
    )
    .into_styled(self.shape_style())
    .draw(target)
  }
  // 四角形
  fn draw_rectangle<D>(
    &self, 
    target : &mut D,
    p      : Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let loc = self.shape_diameter as i32 / 2 ;
    Rectangle::new(
      Point::new(
        p.x - loc,
        p.y - loc,
      ), 
      Size::new(
        self.shape_diameter,
        self.shape_diameter,
      ),
    )
    .into_styled(self.shape_style())
    .draw(target)
  }
}
// 図形スタイル
impl EmbShapegraph { 
  fn shape_style(
    &self
  ) -> PrimitiveStyle<Rgb565>
  {
    match self.shape_mode {
      ShapeMode::RevTriangle |
      ShapeMode::Triangle    |
      ShapeMode::Circle      |
      ShapeMode::Rectangle => { 
        PrimitiveStyleBuilder::new()
          .stroke_color(self.shape_color)
          .stroke_width(self.shape_width)
          .build()
      }
      ShapeMode::FillRevTriangle |
      ShapeMode::FillTriangle    |
      ShapeMode::FillCircle      |
      ShapeMode::FillRectangle => {
        PrimitiveStyleBuilder::new()
          .stroke_color(self.shape_color)
          .stroke_width(self.shape_width)
          .fill_color(self.shape_color)
          .build()
      },
    }
  }
}
// その他セッター
impl EmbShapegraph { 
  pub fn set_shape_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.shape_color = c;
    self
  }

  pub fn set_shape_width(&mut self, c: u32)
    -> &mut Self {
    self.shape_width = c;
    self
  }
 
  pub fn set_shape_diameter(&mut self, c: u32)
    -> &mut Self {
    self.shape_diameter = c;
    self
  }
}