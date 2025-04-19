#![no_std]
use emb_bargraph::*;
use emb_linegraph::*;

// グラフ供給配列数
const GSC   : usize = 1;
// サブ配列数
const SUBGSC: usize = 1;
// グラフ供給種類
type Go1 = EmbLinegraph;
pub enum EmbGraphs {
  Go1(Go1), 
}
impl EmbGraphs {
  // graph_supply_implマクロによる実装
  graph_supply_impl!(
    Go1,
  );
}
// ----------------------------------------
// グラフ供給格納
pub fn graph_supply(
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> (
         [EmbBargraph<'static>; GSC],
         [[EmbGraphs; SUBGSC] ; GSC],
)
{
  let gb  = [graph_box(xsr, ysr, cr, sn),
            ];
  let gos = [graph_obj(&gb[0]),
            ];

  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始, 表示ｻｲｽﾞ  , x目盛,y目盛
    (0, 5) , (320, 240), xsr, ysr,
    //補正率,目盛刻み, 
    cr,      sn, 
    //ﾀｲﾄﾙ
    "qm",
  );
  gb.set_box_color(BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(bg: &EmbBargraph) 
   -> [EmbGraphs; SUBGSC]
{
  // オブジェクト1
  let es = EmbLinegraph::new(bg);
  [EmbGraphs::Go1(es),
  ]
}
// ========================================
// 表示色設定 
const BLACK : Rgb565 = Rgb565::BLACK;
// ----------------------------------------
// graph_supply_implマクロ
#[macro_export]
macro_rules! graph_supply_impl 
{
 ($($go:ident,)*) => {
// --メソッド--
  // データセット
  pub fn set_data(&mut self, x: f32, y: f32) {
    match self {
      $(Self::$go(f) => {f.set_data(x, y);},)*
  }}
  // データリセット
  pub fn reset_data(&mut self) {
    match self {
      $(Self::$go(f) => {
          f.reset_data();
          f.set_shape_color(color_change());
      },)*
  }}
}}
// graph_supply_drawマクロ
#[macro_export]
macro_rules! graph_supply_draw 
{
 ($mobj:ident, 
  $draw_target:ident,
  $($go:ident,)*
 ) => {
  // 描画
  match $mobj {
    $(EmbGraphs::$go(f) =>
      {f.draw(&mut $draw_target).unwrap();},)*
  }
}}
// ----------------------------------------
// カラーチェンジ対応
use core::sync::atomic::{AtomicUsize, Ordering};
static CLCNT: AtomicUsize = 
              AtomicUsize::new(0);

const CLNUM: usize = 15;
const CLTBL: [Rgb565; CLNUM] = [
    Rgb565::CSS_BLUE,
    Rgb565::CSS_CHARTREUSE,
    Rgb565::CSS_CORAL,
    Rgb565::CSS_FUCHSIA,
    Rgb565::CSS_CYAN,
    Rgb565::CSS_GREEN_YELLOW,
    Rgb565::CSS_RED,
    Rgb565::CSS_BLUE_VIOLET,
    Rgb565::CSS_LIGHT_GREEN,
    Rgb565::CSS_KHAKI,
    Rgb565::CSS_MEDIUM_VIOLET_RED,
    Rgb565::CSS_ROYAL_BLUE,
    Rgb565::CSS_TEAL,
    Rgb565::CSS_ORANGE,
    Rgb565::CSS_HOT_PINK,
];

fn color_change() -> Rgb565 {
  let i = CLCNT.fetch_add(
                  1, Ordering::Relaxed
                ) % CLNUM;
  CLTBL[i]
}
// グラフ立体視化オブジェクト
use wio_sbeye::WioSBEye;
use wio_sbcamera::WioSBCamera;
type T = f32;
pub struct Plot3D {
  sb   : WioSBCamera,
  #[allow(clippy::type_complexity)]
  scale: [((T, T, T), (T, T, T)); 9],
  scolr: [Rgb565; 9],
  cr  : (T, T, T),
}
impl Plot3D {
  pub fn new (cr: (T, T, T)) -> Self {
    // カメラ作成
    let mut sb = 
      WioSBCamera::build_default(320., 240.);
    let mut eye = WioSBEye { // カメラ視点
        hw: 0,
        vw: 0,
        r : 350,
        r_limit: 1000,
        step: 10,
    };
    // 視点変更
    eye.right().right().up();
    sb.set_eye(eye.position());// 視点設定
    // グラフ枠設定
    let (x1, x2, y1, y2, z1) = 
      (-20., 330., -20., 280., 120.)
    ;
    Self {sb,
          scale:                // グラフ枠
      [((x1,y1,0.),(x1,y2,0.)), // 縦前左
       ((x1,y1,z1),(x1,y2,z1)), // 縦後左
       ((x2,y1,z1),(x2,y2,z1)), // 縦後右
       ((x1,y1,0.),(x1,y1,z1)), // 奥上左
       ((x1,y2,0.),(x1,y2,z1)), // 奥下左
       ((x2,y2,0.),(x2,y2,z1)), // 奥下右 
       ((x1,y2,0.),(x2,y2,0.)), // 横前下
       ((x1,y2,z1),(x2,y2,z1)), // 横後下
       ((x1,y1,z1),(x2,y1,z1)), // 横後上
      ],
          scolr:                // グラフ枠色
      [Rgb565::WHITE,
       Rgb565::CSS_GRAY,
       Rgb565::CSS_GRAY,
       Rgb565::CSS_LIGHT_GRAY,
       Rgb565::CSS_LIGHT_GRAY,
       Rgb565::CSS_LIGHT_GRAY,
       Rgb565::WHITE,
       Rgb565::CSS_GRAY,
       Rgb565::CSS_GRAY,
      ],
      cr,
    }
  }
}
impl Drawable for Plot3D
{
  type Color = Rgb565;
  type Output = ();

  fn draw<D>(&self, display: &mut D) 
    -> Result<Self::Output, D::Error>
    where
      D: DrawTarget<Color = Rgb565>,
  {
    // グラフ枠領域クリア -------------------
    Rectangle::new(
      Point::new(30,0), Size::new(300, 240)
    ).into_styled(
      PrimitiveStyle::with_fill(Rgb565::BLACK)
    ).draw(display)?;
    // グラフ枠表示 -------------------------
    (0..9).for_each(|i| {
      let (x1, y1) = self.sb.convertf32(
        self.scale[i].0
      );
      let (x2, y2) = self.sb.convertf32(
        self.scale[i].1
      ); 
      let _ = Line::new(
        Point::new(x1 as i32, y1 as i32),
        Point::new(x2 as i32, y2 as i32)
      ).into_styled(
         PrimitiveStyle::with_stroke(
           self.scolr[i], 2
         )
      ).draw(display);
    });
    Ok(())
  }
}
// 立体視変換
impl Plot3D {
  pub fn conv(&self, x:T, y:T, z:T) -> (T, T) 
  {
    let (x1, y1) = self.sb.convertf32((
       x*self.cr.0-20.,
       y*self.cr.1-30., 
       z*self.cr.2
    ));
    (x1/self.cr.0, y1/self.cr.1)
  }
}

