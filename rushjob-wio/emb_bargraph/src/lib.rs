#![no_std]
// 以下のコメント文は、プログラム作る前に
// 設計ぽいことを書いていた形跡だが、
// 実装してく中でかなり変わってしまって
// いるがメンテしてない。
// メンテすんのめんどくさいのと、
// 消すのもしのひないので残してある。
//
// 保持情報
//   表示領域 
//     表示開始位置 Point
//     表示サイズ   Size
//   X目盛数  usize
//   Y目盛数  usize
//   X補正率 f32, Y補正率 f32
//     データが少数点値含む場合に対応
//     ※表示位置がi32のため
//     補正しない場合1.0を設定
//     例
//       0.053(データ) * 1000.0(補正率) = 53
//   目盛原点 Point
//     x = 表示開始位置.x + 30(x余白)
//     y = 表示開始位置.y + 表示サイズ.y -
//         10(y余白)
//   バー幅 = (表示開始位置.x +
//             表示サイズ.x   -
//             目盛原点.x)    /
//            (X目盛数 * 2 + 1)                 
//   X目盛開始位置
//     0位置をバー1個分右にずらす
//     = 目盛原点 + バー幅
//   X位置係数
//     補正後の値のX位置を割り出すための係数
//     = バー幅 * 2  * 補正率
//       使用例
//         X始点 = データ *
//                 X位置係数 -  バー幅 / 2
//         Xサイズ = バー幅
//   Y位置係数
//     = (目盛原点.y - 表示開始位置.y) /
//       Y目盛数  
//       使用例
//         Y始点 = 目盛原点.y -
//                 データ * Y位置係数
//         Yサイズ = データ * Y位置係数 -
//                   1(X目盛軸幅)
//
// あと、本当はやっておかないといけない事
//   バーが表示領域をはみ出すようなデータが
//   来た時に、バー表示の調整を行う。
//   →どっかのタイミングで出来たらやる。
//     データに気をつけておけば問題ない。
//     →Yに対してのみ対応、
//       Xはほっといても大丈夫だろう。

pub use core::ops::Range;

pub use embedded_graphics::{
  mono_font::{
  ascii::FONT_6X12,
    MonoTextStyleBuilder,
    MonoTextStyle,
  },
  primitives::{
    Rectangle,
    PrimitiveStyle,
    PrimitiveStyleBuilder,
    Polyline,
  },
  pixelcolor::Rgb565,
  prelude::*,
};

use embedded_plots::axis::{
  Axis, Placement, Scale
};

const SCAL_THICK: i32 = 1;
const BOX_THICK : i32 = 1;
const X_MARGIN  : i32 = 30;
const Y_MARGIN  : i32 = 26;

#[derive(Debug, PartialEq, Eq)]
enum DrawMode {
  AllClear,
  Clear,
  Scale,
  Data,
}

pub struct EmbBargraph<'a> {
  dsp_start     : Point,     // 表示開始位置
  dsp_size      : Size,      // 表示サイズ
  x_scale_range : Range<i32>,// X目盛レンジ
  y_scale_range : Range<i32>,// Y目盛レンジ
  scale_nik     : (usize, usize), // 目盛刻み
  scale_start   : Point,     // 目盛原点
  bar_width     : i32,       // バー幅
  x_scale_start : i32,       // X目盛開始位置
  correct_shift : (f32, f32),// 位置シフ(x,y)
  correct_fact  : (f32, f32),// 位置係数(x,y)
  text_style    : MonoTextStyle<'a, Rgb565>,
  bar_color     : Rgb565,    // バー色
  base_color    : Rgb565,    // 背景色
  text_color    : Rgb565,    // 文字色
  scale_color   : Rgb565,    // 目盛色
  box_color     : Rgb565,    // 表示枠色
  title         : &'a str,   // タイトル
  draw_mode     : DrawMode,  // 描画モード
  data          : (f32, f32),// データ
}

impl<'a> EmbBargraph<'a> {
  pub fn new(
    dspstart      : (i32, i32), 
    dspsize       : (u32, u32), 
    x_scale_range : Range<i32>,
    y_scale_range : Range<i32>,
    correct_rate  : (f32, f32),
    scale_nik     : (usize, usize),
    title         : &'a str,
  ) -> Self
  {
    // 表示開始位置
    let dsp_start = Point::new(
      dspstart.0,
      dspstart.1,
    );
    // 表示サイズ
    let dsp_size  = Size::new(
      dspsize.0,
      dspsize.1,
    );
    // 目盛原点
    let scale_start = Point::new(
      dsp_start.x + X_MARGIN,
      dsp_start.y - Y_MARGIN + 
      dsp_size.height as i32,
    );
    // バー幅
    let mut bar_width = 
      (dsp_start.x           +
       dsp_size.width as i32 - 
       scale_start.x)        
      /
      (x_scale_range.len() as i32 * 2 + 1);
    if bar_width < 1 { bar_width = 1; }
    // X目盛開始位置
    let x_scale_start = 
      scale_start.x + bar_width;
    // 位置量
    let correct_val =
    ( // X量
      (dsp_start.x +
       dsp_size.width as i32 -
       x_scale_start)     as f32 /
      x_scale_range.len() as f32,
      // Y量
      (scale_start.y -  
       dsp_start.y)       as f32 / 
      y_scale_range.len() as f32,
    );
    // シフト
    let correct_shift =
    ( //Xシフト
      x_scale_range.start as f32 *
      correct_val.0,
      // Yシフト
      y_scale_range.start as f32 *
      correct_val.1,
    );
    // 補正率
    let correct_fact = 
    (
      // X補正率
      correct_val.0 * correct_rate.0,
      // Y補正率
      correct_val.1 * correct_rate.1,
    );
    // 色設定
    let bar_color   = Rgb565::YELLOW;
    let base_color  = Rgb565::BLACK;
    let text_color  = Rgb565::WHITE;
    let scale_color = Rgb565::WHITE;
    let box_color   = Rgb565::WHITE;
    // 目盛文字書式
    let text_style = 
      MonoTextStyleBuilder::new()
        .font(&FONT_6X12)
        .text_color(text_color)
        .build();

    let draw_mode = DrawMode::AllClear;

    let data = (0.0, 0.0);

    EmbBargraph {
      dsp_start,
      dsp_size,
      x_scale_range,
      y_scale_range,
      scale_nik,
      scale_start,
      bar_width,
      x_scale_start,
      correct_shift,
      correct_fact,
      text_style,
      bar_color,
      base_color,
      text_color,
      scale_color,
      box_color,
      title,
      draw_mode,
      data,
    }
  }
}
// 描画モード、データセット
impl<'a> EmbBargraph<'a> {
  pub fn mode_allclear(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::AllClear;
    self
  }

  pub fn mode_clear(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::Clear;
    self
  }

  pub fn mode_scale(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::Scale;
    self
  }
    
  pub fn mode_data(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::Data;
    self
  }

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
impl<'a> Drawable for EmbBargraph<'a>
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    match self.draw_mode  {
      DrawMode::AllClear => {
        self.draw_allclear(display)?;
      },
      DrawMode::Clear    => {
        self.draw_clear(display)?;
      },
      DrawMode::Scale    => {
        self.draw_scale_x(display)?;
        self.draw_scale_y(display)?;
        self.draw_scale_box(display)?;
      }, 
      DrawMode::Data      => {
        self.draw_data(display)?;
      }, 
    }
    Ok(())
  }
}
// 描画ヘルパー関数
impl<'a> EmbBargraph<'a> {
  // バー表示
  fn draw_data<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    // バーサイズ
    let ysize =
      (self.data.1 * self.correct_fact.1 +
       self.correct_shift.1 
       ) as i32;

    let mut s = Size::new(
      self.bar_width as u32,
      (
        if ysize <= 0 {
          0
        } else {
          ysize - BOX_THICK
        }
      ) as u32,
    );
    // バー表示開始ポイント
    let mut p = Point::new(
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

    // バー領域を超える場合、バー高幅を固定
    if p.y < self.dsp_start.y + BOX_THICK {
      p.y = self.dsp_start.y + 
            BOX_THICK;
      s.height = (
        self.scale_start.y - 
        self.dsp_start.y   -
        SCAL_THICK         -
        BOX_THICK
      ) as u32;
    }
    // 表示
    Rectangle::new(p, s)
    .into_styled(
      PrimitiveStyle::with_fill(
        self.bar_color
      )
    )
    .draw(target)
  }

  // 描画領域クリア
  fn draw_allclear<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let style = PrimitiveStyleBuilder::new()
      .stroke_color(self.box_color)
      .stroke_width(BOX_THICK as u32)
      .fill_color(self.base_color)
      .build();

    Rectangle::new(
      self.dsp_start, 
      self.dsp_size,
    ) 
    .into_styled(style)
    .draw(target)
  }

  // バー領域クリア
  fn draw_clear<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    
    Rectangle::new(
      Point::new(
        self.scale_start.x + SCAL_THICK + 2,
        self.dsp_start.y   + BOX_THICK,
      ), 
      Size::new(
        (
          self.dsp_start.x           +
          self.dsp_size.width as i32 -
          self.scale_start.x         - 
          SCAL_THICK - 2             -
          BOX_THICK
        ) as u32,
        (
          self.scale_start.y - 
          self.dsp_start.y   -
          SCAL_THICK         - 
          BOX_THICK
        ) as u32,
      ),
    )
    .into_styled(
      PrimitiveStyle::with_fill(
        self.base_color
      )
    )
    .draw(target)
  }

  // 目盛表示X軸
  fn draw_scale_x<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    Axis::new(
      self.x_scale_range.clone()
    )
    .set_title(self.title)
    .set_scale(
      Scale::Fixed(self.scale_nik.0)
    )
    .into_drawable_axis(
      Placement::X {
        x1: self.x_scale_start,
        x2: self.dsp_start.x           +
            self.dsp_size.width as i32 -
            BOX_THICK,
        y : self.scale_start.y,
      }
    )
    .set_color(self.scale_color)
    .set_text_style(self.text_style)
    .set_thickness(SCAL_THICK as usize)
    .set_tick_size(2)
    .draw(target)
  }
 
  // 目盛枠補完
  fn draw_scale_box<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let style = PrimitiveStyleBuilder::new()
      .stroke_color(self.scale_color)
      .stroke_width(BOX_THICK as u32)
      .build();

    Polyline::new(
     &[
        Point::new(
          self.scale_start.x,
          self.dsp_start.y,
        ),
        Point::new(
          self.dsp_start.x           +
          self.dsp_size.width as i32,
          self.dsp_start.y,
        ),
        Point::new(
          self.dsp_start.x           +
          self.dsp_size.width as i32,
          self.scale_start.y,
        ),
      ]
    )
    .into_styled(style)
    .draw(target)
  }

  // 目盛表示Y軸
  fn draw_scale_y<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    Axis::new(
      self.y_scale_range.clone()
    )
//  .set_title("Y")
    .set_scale(
      Scale::Fixed(self.scale_nik.1)
    )
    .into_drawable_axis(
      Placement::Y {
        y1: self.dsp_start.y,
        y2: self.scale_start.y,
        x : self.scale_start.x,
      }
    )
    .set_color(self.scale_color)
    .set_text_style(self.text_style)
    .set_thickness(SCAL_THICK as usize)
    .set_tick_size(2)
    .draw(target)
  }
}
// その他セッター
impl<'a> EmbBargraph<'a> {
  pub fn set_bar_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.bar_color = c;
    self
  }

  pub fn set_base_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.base_color = c;
    self
  }

  pub fn set_text_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.text_color = c;
    self.text_style = 
      MonoTextStyleBuilder::new()
        .font(&FONT_6X12)
        .text_color(self.text_color)
        .build();
    self
  }

  pub fn set_scale_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.scale_color = c;
    self
  }

  pub fn set_box_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.box_color = c;
    self
  }
}
// その他ゲッター
impl<'a> EmbBargraph<'a> {
  // 表示開始位置
  pub fn dsp_start(&self) -> Point {
    self.dsp_start.clone()
  }
  // 表示サイズ
  pub fn dsp_size(&self)  -> Size {
    self.dsp_size.clone()
  }
  // X目盛レンジ
  pub fn x_scale_range(&self) -> Range<i32> {
    self.x_scale_range.clone()
  }
  // Y目盛レンジ
  pub fn y_scale_range(&self) -> Range<i32> {
    self.y_scale_range.clone()
  }
  // 目盛原点
  pub fn scale_start(&self) -> Point {
    self.scale_start.clone()
  }
  // バー幅
  pub fn bar_width(&self)  -> i32 {
    self.bar_width
  }
  // X目盛開始位置
  pub fn x_scale_start(&self) -> i32 {
    self.x_scale_start
  }
  // 位置シフト(x,y)
  pub fn correct_shift(&self) -> (f32, f32) {
    self.correct_shift
  }
  // 位置係数(x,y)
  pub fn correct_fact(&self) -> (f32, f32) {
    self.correct_fact
  }
  // データ(x,y) デバッグ用
  pub fn data(&self) -> (f32, f32) {
    self.data
  }
}

