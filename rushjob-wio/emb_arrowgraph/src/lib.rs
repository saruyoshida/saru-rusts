// 矢印グラフ
#![no_std]
pub use embedded_graphics::{
  primitives::{Triangle, Line, Circle},
};
#[allow(unused_imports)]
use num_traits::Float;

use emb_bargraph::*;
use emb_linetrim::*;

type T=f32;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ShapeMode {
  Arrow,
  FillArrow,
}
#[derive(Clone)]
pub struct EmbArrowgraph {
  scale_start   : Point,     // 目盛原点
  bar_width     : i32,       // バー幅
  x_scale_start : i32,       // X目盛開始位置
  draw_area     : Rectangle, // draw領域
  correct_shift : (T, T),    // 位置ｼﾌﾄ(x,y)
  correct_fact  : (T, T),    // 位置係数(x,y)
  shape_mode    : ShapeMode, // 図形モード
  shape_color   : Rgb565,    // 図形色
  shape_width   : u32,       // 図形線幅
  shape_diameter: u32,       // 図形直径
  norm          : T,         // norm
  start         : (T, T),    // 始点
  end           : (T, T),    // 終点
  draw_th       : (T, T),    // 描画閾値
  color_th      : (T, T),    // 色閾値
  h             : T,         // 羽の長さ
  blade         : Triangle,  // 羽
  tail          : Line,      // シッポ
  iscolormap    : bool,      // ｶﾗｰﾏｯﾌﾟ設定
  iszeronodraw  : bool,      // zero値描画設定
  elt           : EmbLineTrim, 
}
// new
impl EmbArrowgraph {
  pub fn new(
    graph     : &EmbBargraph, // 棒グラフ
    iscolormap: bool,         // ｶﾗｰﾏｯﾌﾟ適用
  ) -> Self
  { // LineTrim調整
    let mut area = graph.draw_area();
    area.top_left.x -= 2;
    area.size.width += 2;
    // EmbArrowgraph作成
    EmbArrowgraph {
      scale_start   : graph.scale_start(),
      bar_width     : graph.bar_width(),
      x_scale_start : graph.x_scale_start(),
      draw_area     : graph.draw_area(),
      correct_shift : graph.correct_shift(),
      correct_fact  : graph.correct_fact(),
      shape_mode    : ShapeMode::Arrow,
      shape_color   : Rgb565::WHITE,
      shape_width   : 1,
      shape_diameter: 2,
      norm          : 1.,
      start         : (0., 0.),
      end           : (0., 0.),
      draw_th       : (0.0001, 1000.),
      color_th      : (0., 0.5),
      h             : 4.,
      blade         : Triangle::new(
                        Point::new(0, 0),
                        Point::new(0, 0),
                        Point::new(0, 0),
                      ),
      tail          : Line::new(
                        Point::new(0, 0),
                        Point::new(0, 0),
                      ),
      iscolormap,
      iszeronodraw  : true,
      elt           : EmbLineTrim::new(
                        area
                      ),
    }
  }
}
// 図形モード、データセット
impl EmbArrowgraph {
  // 矢印塗りつぶしなし
  pub fn mode_arrow(&mut self) -> &mut Self {
    self.shape_mode = ShapeMode::Arrow;
    self
  }
  // 矢印塗りつぶしあり
  pub fn mode_fillarrow(&mut self)
  -> &mut Self {
    self.shape_mode = ShapeMode::FillArrow;
    self
  }
  //データセット※空定義
  pub fn set_data(
    &mut self,
    _x : T,
    _y : T,
  ) -> &mut Self
  {self}
  //データセット
  pub fn set_data_arrow(
    &mut self, 
    data : ((T, T), (T, T)),
    norm : T,
  ) -> &mut Self {
    self.start = data.0;
    self.end   = data.1;
    self.norm  = norm;
    // 始点Aを求める
    let a = self.set_point(
      self.start.0, self.start.1
    );
    // 終点Bを求める
    let b = self.set_point(
      self.end.0, self.end.1
    );
    // 矢印を作成する
    self.set_arrowpoint(a, b);
    // カラーマップを設定する
    if self.iscolormap {self.set_colormap();}
    self
  }
  // 矢印作成
  fn set_arrowpoint(
    &mut self,
    a  : (T, T),
    b  : (T, T),
  ) {
    // 矢印を作る
    let vx = b.0 - a.0;
    let vy = b.1 - a.1;
    let v = (vx*vx + vy*vy).sqrt();
    let (lx, ly, rx, ry) = {
      let h = if self.h < v {
        self.h
      } else {
        self.h * v / self.h 
      };
      let (ux, uy) = (vx/v, vy/v);

      let w = h * 0.4;
      (b.0 - uy*w - ux*h,
       b.1 + ux*w - uy*h,
       b.0 + uy*w - ux*h,
       b.1 - ux*w - uy*h,
      )
    };
    let top = Point::new(
      b.0 as i32, b.1 as i32
    );
    // シッポ
    self.tail = Line::new(
      top,
      Point::new(a.0 as i32, a.1 as i32),
    );
    // 羽
    self.blade = Triangle::new(
      top,
      Point::new(lx as i32, ly as i32),
      Point::new(rx as i32, ry as i32),
    );
  }
  // 表示ポイント設定
  fn set_point(&mut self, x: T, y: T) 
  -> (T, T) {
    // サイズ
    let ysize = y * self.correct_fact.1 -
                self.correct_shift.1 
    ;
    // ポイント
    (self.x_scale_start as T + 
     x * self.correct_fact.0 -
     self.correct_shift.0    -
     self.bar_width as T / 2.
     ,
     self.scale_start.y as T - ysize
    )
  }
}
// 描画
impl Drawable for EmbArrowgraph
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    if self.iszeronodraw &&
       (self.norm > self.draw_th.1 ||
        self.norm < self.draw_th.0) {
      return Ok(());
    } 
    // 図形表示
    match self.shape_mode {
      ShapeMode::Arrow |
      ShapeMode::FillArrow => {
        self.draw_arrow(display)?;
      },
    };
    Ok(())
  }
}
// 図形表示
impl EmbArrowgraph {
  fn draw_arrow<D>(
    &self, 
    target: &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    // シッポdraw範囲設定
    let mut pstart = self.tail.start;
    let mut pend   = self.tail.end;
    if !self.elt.line_trim(
      &mut pstart, 
      &mut pend
    ) {
      return Ok(());
    }
    // シッポ描画
    Line::new(pstart, pend)
    .into_styled(self.shape_style())
    .draw(target)?;
    // 羽draw領域外チェック
    if self.draw_area
           .contains(self.tail.start) {
      // 羽描画
      self.blade.into_styled(
        self.shape_style()
      ).draw(target)?;
    }
    Ok(())
  }
}
// 図形スタイル
impl EmbArrowgraph {
  fn shape_style(
    &self
  ) -> PrimitiveStyle<Rgb565>
  {
    match self.shape_mode {
      // 塗りつぶしなし
      ShapeMode::Arrow => {
        PrimitiveStyleBuilder::new()
          .stroke_color(self.shape_color)
          .stroke_width(self.shape_width)
          .build()
      }
      // 塗りつぶしあり
      ShapeMode::FillArrow => {
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
impl EmbArrowgraph {
  // 色設定
  pub fn set_shape_color(&mut self, c: Rgb565)
  -> &mut Self {
    self.shape_color = c;
    self
  }
  // 太さ設定
  pub fn set_shape_width(&mut self, c: u32)
  -> &mut Self {
    self.shape_width = c;
    self
  }
  // 半径設定
  pub fn set_shape_diameter(&mut self, c: u32)
  -> &mut Self {
    self.shape_diameter = c;
    self
  }
  // 描画閾値設定
  pub fn set_draw_th(&mut self, t: (T, T))
  -> &mut Self {
    self.draw_th = t;
    self
  }
  // 色閾値設定
  pub fn set_color_th(&mut self, t:(T, T))
  -> &mut Self {
    self.color_th = t;
    self
  }
  // 羽の長さ
  pub fn set_h(&mut self, t: T)
  -> &mut Self {
    self.h = t;
    self
  }
  // リセット(空定義)
  pub fn reset_data(&mut self) -> &mut Self {
    self
  }
}
// カラーマップ
// 0.0〜1.0の正規化された値が渡されてくる
// ことを想定し16分割
const CLNUM: usize = 16;
const CLTBL: [Rgb565; CLNUM] = [
    Rgb565::CSS_MIDNIGHT_BLUE,
    Rgb565::CSS_DARK_SLATE_BLUE,
    Rgb565::CSS_ROYAL_BLUE,
    Rgb565::CSS_STEEL_BLUE,
    Rgb565::CSS_MEDIUM_TURQUOISE,
    Rgb565::CSS_CYAN,
    Rgb565::CSS_CHARTREUSE,
    Rgb565::CSS_LIME,
    Rgb565::CSS_GREEN_YELLOW,
    Rgb565::CSS_YELLOW,
    Rgb565::CSS_GOLDENROD,
    Rgb565::CSS_ORANGE,
    Rgb565::CSS_DARK_ORANGE,
    Rgb565::CSS_HOT_PINK,
    Rgb565::CSS_ORANGE_RED,
    Rgb565::CSS_RED,
];
impl EmbArrowgraph {
  fn set_colormap(&mut self) {
    // 正規化
    let i = 
      (self.norm - self.color_th.0) / 
      (self.color_th.1 - self.color_th.0)
    ;
    // index=v/閾値/(1/色数(16))の切捨て
    let i = (i / (1. / CLNUM as f32)) 
            as usize;
    // index>=色数の場合、index=色数-1
    self.shape_color = if i >= CLNUM {
                         CLTBL[CLNUM-1]
                       } else {
                         CLTBL[i]
                       }
    ;
  }
  // ｶﾗｰﾏｯﾌﾟ有効化
  pub fn colormap_on(&mut self)
  -> &mut Self {
    self.iscolormap = true;
    self
  }
  // ｶﾗｰﾏｯﾌﾟ無効化
  pub fn colormap_off(&mut self)
  -> &mut Self {
    self.iscolormap = false;
    self
  }
}
// 関連関数
impl EmbArrowgraph {
  // 引数作成:2d可変長
  #[allow(clippy::type_complexity)]
  pub fn make_argument2d(
    xy  : (T, T),
    uv  : (T, T),
    base: T
  ) -> (((T, T), (T, T)), T) {
    ((xy,
      (uv.0*base + xy.0, uv.1*base + xy.1)
     ),
     (uv.0*uv.0 + uv.1*uv.1).sqrt()
    )
  }
  // 引数作成:2d固定長
  #[allow(clippy::type_complexity)]
  pub fn make_argument2d_fixed(
    xy  : (T, T),
    uv  : (T, T),
    base: T
  ) -> (((T, T), (T, T)), T) {
    let n = (uv.0*uv.0 + uv.1*uv.1).sqrt();
    ((xy,
      (uv.0/n*base + xy.0, uv.1/n*base + xy.1)
     ),
     n
    )
  }
  // 引数作成:3d可変長
  #[allow(clippy::type_complexity)]
  pub fn make_argument3d(
    xyz  : (T, T, T),
    uvw  : (T, T, T),
    base : T
  ) -> (((T, T, T), (T, T, T)), T) {
    ((xyz,
      (uvw.0*base + xyz.0, 
       uvw.1*base + xyz.1,
       uvw.2*base + xyz.2)
     ),
     (uvw.0*uvw.0 + 
      uvw.1*uvw.1 +
      uvw.2*uvw.2
     ).sqrt()
    )
  }
  // 引数作成:3d固定長
  #[allow(clippy::type_complexity)]
  pub fn make_argument3d_fixed(
    xyz  : (T, T, T),
    uvw  : (T, T, T),
    base : T
  ) -> (((T, T, T), (T, T, T)), T) {
    let n = (uvw.0*uvw.0 + 
             uvw.1*uvw.1 +
             uvw.2*uvw.2).sqrt();
    ((xyz,
      (uvw.0*base/n + xyz.0, 
       uvw.1*base/n + xyz.1,
       uvw.2*base/n + xyz.2)
     ),
     n
    )
  }
}
