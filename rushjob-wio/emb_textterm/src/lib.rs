#![no_std]

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
  },
  pixelcolor::Rgb565,
  prelude::*,
  text::Text,
};

pub use heapless::String;

pub type EttString = String<256>;

const FONT_WIDTH: i32  =     // フォント幅
  FONT_6X12.character_size.width  as i32;
const FONT_HEIGHT: i32 =     // フォント高
  FONT_6X12.character_size.height as i32;
const FONT_SPACE : i32 =
  FONT_6X12.character_spacing as i32;
const BOX_THICK : i32 = 1;
const X_MARGIN  : i32 = 2;
const Y_MARGIN  : i32 = 2;

#[derive(Debug, PartialEq, Eq)]
enum DrawMode {
  AllClear,
  Clear,
  Data,
}

pub struct EmbTextterm<'a> {
  dsp_start     : Point,     // 表示開始位置
  dsp_size      : Size,      // 表示サイズ
  cursor        : Point,     // 文字表示位置
  cursor_next   : Point,     // 次回位置
  cursor_min    : Point,     // 文字位置最小
  cursor_max    : Point,     // 文字位置最大
  text_style    : MonoTextStyle<'a, Rgb565>,
  base_color    : Rgb565,    // 背景色
  text_color    : Rgb565,    // 文字色
  txt2_color    : Rgb565,    // 次文字色
  box_color     : Rgb565,    // 表示枠色
  draw_mode     : DrawMode,  // 描画モード
  data          : EttString,   // データ
}

impl<'a> EmbTextterm<'a> {
  pub fn new(
    dspstart      : (i32, i32), 
    dspsize       : (u32, u32), 
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
    // 色設定
    let base_color  = Rgb565::BLACK;
    let text_color  = Rgb565::WHITE;
    let box_color   = Rgb565::YELLOW;
    let txt2_color  = Rgb565::YELLOW;
    // 文字書式
    let text_style = 
      MonoTextStyleBuilder::new()
        .font(&FONT_6X12)
        .text_color(text_color)
        .build();
    // 文字位置最少
    let cursor_min = Point::new(
      dsp_start.x +
      X_MARGIN    +
      BOX_THICK,
      dsp_start.y +
      FONT_HEIGHT +
      Y_MARGIN
    );
    // 文字位置最大
    let cursor_max = Point::new(
      dsp_start.x            +
      dsp_size.width as i32  -
      FONT_WIDTH             -
      X_MARGIN               -
      BOX_THICK * 2,
      dsp_start.y            +
      dsp_size.height as i32 -
      Y_MARGIN               -
      BOX_THICK * 2,
    );
    // 文字表示初期位置
    let cursor      = cursor_min.clone();
    let cursor_next = cursor.clone();
    // 描画モード初期値
    let draw_mode = DrawMode::AllClear;
    // 文字列初期値
    let data = EttString::new();

    EmbTextterm {
      dsp_start,
      dsp_size,
      cursor,
      cursor_next,
      cursor_min,
      cursor_max,
      text_style,
      base_color,
      text_color,
      txt2_color,
      box_color,
      draw_mode,
      data,
    }
  }
}

impl<'a> EmbTextterm<'a> {
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

  pub fn mode_reset(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::Clear;
    self.cursor    = self.cursor_min.clone();
    self
  }
    
  pub fn mode_data(&mut self) -> &mut Self
  {
    self.draw_mode = DrawMode::Data;
    self
  }

  pub fn set_data(
    &mut self,
    s: EttString,
  ) -> &mut Self
  {
    self.data = s;
    self.cursor_swap();
    self
  }

  fn cursor_swap(&mut self)
  {// 予め次の表示位置を計算しておく
    self.cursor = self.cursor_next.clone();
    let mut cursor = &mut self.cursor_next;
    
    for c in self.data.as_str().chars() {
      // x位置がオーバー または 改行
      if cursor.x >= self.cursor_max.x || 
         c == '\n' 
      {
        cursor.x = self.cursor_min.x;
        cursor.y = cursor.y +
                   FONT_HEIGHT;
        // y位置がオーバー
        if cursor.y >= self.cursor_max.y {
          *cursor = self.cursor_min.clone();
          // テキスト色入替え
          self.text_style.text_color =
            Some(self.txt2_color);

          core::mem::swap(
            &mut self.text_color,
            &mut self.txt2_color,
          );
        }
      }
      // 改行でなければx位置シフト
      if c != '\n' {
        // x位置シフト
        cursor.x += FONT_WIDTH +
                    FONT_SPACE;
      }
    }
  }
}

impl<'a> Drawable for EmbTextterm<'a>
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
      DrawMode::Data      => {
        self.draw_data(display)?;
      }, 
    }
    Ok(())
  }
}

impl<'a> EmbTextterm<'a> {
  // 文字表示
  fn draw_data<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    let mut cursor = self.cursor.clone();
    for c in self.data.as_str().chars() {
      // x位置がオーバー または 改行
      if cursor.x >= self.cursor_max.x || 
         c == '\n' 
      {
        cursor.x = self.cursor_min.x;
        cursor.y = cursor.y +
                   FONT_HEIGHT;
        // y位置がオーバー
        if cursor.y >= self.cursor_max.y {
          cursor = self.cursor_min.clone();
        }
        // 次行クリア
        self.draw_lineclear(target, &cursor)?;
      }
      // 改行でなければ表示
      if c != '\n' {
        let mut buf = [0u8; 8];
        Text::new(
          c.encode_utf8(&mut buf), 
          cursor, 
          self.text_style
        )
        .draw(target)?;
        // x位置シフト
        cursor.x += FONT_WIDTH +
                    FONT_SPACE;
      }
    }
    Ok(())
  }
  // 文字表示行クリア
  fn draw_lineclear<D>(
    &self, 
    target : &mut D,
    cursor : &Point,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    Rectangle::new(
      Point::new(
        self.dsp_start.x +
        BOX_THICK,
        cursor.y         - 
        FONT_HEIGHT      +
        1,
      ), 
      Size::new(
        (
          self.dsp_size.width as i32  -
          BOX_THICK * 2
        ) as u32,
        (FONT_HEIGHT + 4) as u32,
      ),
    )
    .into_styled(
      PrimitiveStyle::with_fill(
        self.base_color
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
  // 文字領域クリア
  fn draw_clear<D>(
    &self, 
    target : &mut D,
  ) -> Result<(), D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    
    Rectangle::new(
      Point::new(
        self.dsp_start.x + BOX_THICK,
        self.dsp_start.y + BOX_THICK,
      ), 
      Size::new(
        (
          self.dsp_size.width as i32  -
          BOX_THICK * 2
        ) as u32,
        (
          self.dsp_size.height as i32 -
          BOX_THICK * 2
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
}

impl<'a> EmbTextterm<'a> {
  pub fn set_base_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.base_color = c;
    self
  }

  pub fn set_text_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.text_color = c;
    self.text_style.text_color = Some(c);
    self
  }

  pub fn set_txt2_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.txt2_color = c;
    self
  }
  
  pub fn set_box_color(&mut self, c: Rgb565)
    -> &mut Self {
    self.box_color = c;
    self
  }
}

