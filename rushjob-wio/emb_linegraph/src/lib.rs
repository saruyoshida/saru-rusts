#![no_std]

use micromath::F32Ext;
use emb_bargraph::*;
use emb_linetrim::*;

pub use embedded_graphics::{
  primitives::{
    Line,
    Circle,
  },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ShapeMode {
  RealLine,
  DotLine,
}
#[derive(Clone)]
pub struct EmbLinegraph {
  scale_start   : Point,     // 目盛原点
  bar_width     : i32,       // バー幅
  x_scale_start : i32,       // X目盛開始位置
  correct_shift : (f32, f32),// 位置シフ(x,y)
  correct_fact  : (f32, f32),// 位置係数(x,y)
  shape_mode    : ShapeMode, // 図形モード
  shape_color   : Rgb565,    // 図形色
  shape_width   : u32,       // 図形線幅
  dot_interval  : usize,     // 点線間隔
  data          : (f32, f32),// データ
  shape_start   : Point,     // 開始位置
  shape_end     : Point,     // 終了位置
  elt           : EmbLineTrim, 
}
// new
impl EmbLinegraph {
  pub fn new(
    graph : &EmbBargraph,    // 棒グラフ
  ) -> Self
  {
    // 調整
    let mut area = graph.draw_area();
    area.top_left.x -= 2;
    area.size.width += 2;
    // 調整終わり
    EmbLinegraph {
      scale_start   : graph.scale_start(),
      bar_width     : graph.bar_width(),
      x_scale_start : graph.x_scale_start(),
      correct_shift : graph.correct_shift(),
      correct_fact  : graph.correct_fact(),
      shape_mode    : ShapeMode::RealLine,
      shape_color   : Rgb565::RED,
      shape_width   : 1,
      dot_interval  : 1,
      data          : (0.0, 0.0),
      shape_start   : Point::new(
                        core::i32::MAX,
                        core::i32::MAX
                      ),
      shape_end     : Point::new(
                        core::i32::MAX,
                        core::i32::MAX
                      ),
      elt           : EmbLineTrim::new(
                        area
                      ),
    }
  }
}
// 図形モード、データセット
impl EmbLinegraph {
  // 実線
  pub fn mode_realline(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::RealLine;
    self
  }
  // 点線
  pub fn mode_dotline(
    &mut self
  ) -> &mut Self
  {
    self.shape_mode = ShapeMode::DotLine;
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
    self.set_point();
    self
  }
  // 表示ポイント設定
  fn set_point(&mut self) {
    core::mem::swap(
      &mut self.shape_start,
      &mut self.shape_end,
    );
    // サイズ
    let ysize =
      (self.data.1 * self.correct_fact.1 -
       self.correct_shift.1 
       ) as i32;
    // 表示ポイント
    self.shape_end = Point::new(
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
  }
  // リセット
  pub fn reset_data(&mut self) -> &mut Self {
    self.shape_start.x = core::i32::MAX;
    self.shape_start.y = core::i32::MAX;
    self.shape_end.x   = core::i32::MAX;
    self.shape_end.y   = core::i32::MAX;

    self
  }
}
// 描画
impl Drawable for EmbLinegraph
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { 
    // 初回の場合、描画スキップ
    if self.shape_start.x == 
       core::i32::MAX {
      return Ok(());
    }
    // 表示枠内チェック&調整
    let mut pstart = self.shape_start.clone();
    let mut pend   = self.shape_end.clone();
    if !self.elt.line_trim(
      &mut pstart, 
      &mut pend
    ) 
    {
      return Ok(());
    }
    // 実線の場合
    if self.shape_mode == ShapeMode::RealLine 
    {
      // 実線描画
      Line::new(
        pstart,
        pend
      )
      .into_styled(self.shape_style())
      .draw(display)
    } else {
      // 点線描画
      self.draw_dotline(display, pstart, pend)
    }
  }
}
// 点線描画
impl EmbLinegraph {
  fn draw_dotline<D>(
    &self,
    target      : &mut D,
    shape_start : Point,
    shape_end   : Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  { // 終点と始点から長さと角度を算出
    let d   = shape_end - shape_start;
    let len = ((d.x * d.x + d.y * d.y) as f32)
              .sqrt();
    let rad = (d.y as f32).atan2(d.x as f32);

    let mut dotis = true;
    // 点線描画
    for i in (0..len as i32)
             .step_by(self.dot_interval) {
      if dotis {
        Circle::new(
          Point::new(
            shape_start.x + 
            (i as f32 * rad.cos())  as i32,
            shape_start.y + 
            (i as f32 * rad.sin()) as i32,
          ),
          self.shape_width,
        )
        .into_styled(self.shape_style())
        .draw(target)?;
      }      
      dotis = !dotis;
    }
    Ok(())
  }
}
// 図形スタイル
impl EmbLinegraph {
  fn shape_style(
    &self
  ) -> PrimitiveStyle<Rgb565>
  {     
    match self.shape_mode {
      ShapeMode::RealLine => { 
        PrimitiveStyleBuilder::new()
          .stroke_color(self.shape_color)
          .stroke_width(self.shape_width)
          .build()
      },
      ShapeMode::DotLine  => {
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
impl EmbLinegraph {
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

  pub fn set_dot_interval(&mut self, c: usize)
    -> &mut Self {
    self.dot_interval = c;
    self
  }
}
