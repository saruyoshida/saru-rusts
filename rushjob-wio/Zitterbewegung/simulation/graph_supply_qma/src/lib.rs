#![no_std]
use emb_bargraph::*;
use emb_shapegraph::*;
// グラフ供給配列数
const GSC   : usize = 4;
// サブ配列数
const SUBGSC: usize = 4;
// グラフ供給種類
type Go1 = EmbShapegraph;
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
#[allow(clippy::too_many_arguments)]
pub fn graph_supply(
  xsr : Range<i32>,
  ysr : Range<i32>,
  zsr : Range<i32>,
  cr  : (f32, f32, f32),
  sn  : (usize, usize, usize),
  sxsr: Range<i32>,
  sysr: Range<i32>,
  slcr: (f32, f32),
  slsn: (usize, usize),
) -> (
  [EmbBargraph<'static>; GSC],
  [EmbGraphs; SUBGSC],
)
{
  let gb  = [
    graph_box( // x-y平面
      0  , 2,   1, xsr.clone(), ysr.clone(),
                  (cr.0, cr.1), (sn.0, sn.1),
    ),
    graph_box( // z-y平面
      124, 2,   2, zsr.clone(), ysr.clone(), 
                  (cr.1, cr.2), (sn.1, sn.2),
    ),
    graph_box( // x-z平面
      0  , 122, 3, xsr, zsr, 
                  (cr.0, cr.2), (sn.0, sn.2),
    ),
    graph_box( // スライス
      124, 122, 4, sxsr, sysr, 
      (slcr.0, slcr.1), (slsn.0, slsn.1),
    ),
  ];
  let gos = [
    graph_obj(&gb[0]),
    graph_obj(&gb[1]),
    graph_obj(&gb[2]),
    graph_obj(&gb[3]),
  ];
  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  x: i32, y: i32, title_no: usize,
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ,x目盛,y目盛,補正率,目盛
    (x, y)  , (122, 118), xsr, ysr, cr, sn,
    //ﾀｲﾄﾙ
    match title_no {
      1 => "x-y",
      2 => "z-y",
      3 => "x-z",
      _ => "slice",
    }
  );
  gb.set_box_color(Rgb565::BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(bg: &EmbBargraph) 
   -> EmbGraphs
{
  // オブジェクト1
  let mut es = EmbShapegraph::new(bg); 
          es.set_shape_diameter(1)
            .mode_fillcircle();

  EmbGraphs::Go1(es)
}
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
  // カラーセット
  pub fn set_color(&mut self, c: Rgb565) {
    match self {
      $(Self::$go(f) => {
          f.set_shape_color(c);
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
use core::sync::atomic
              ::{AtomicUsize, Ordering};
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

pub fn color_change() -> Rgb565 {
  let i = CLCNT.fetch_add(
                  1, Ordering::Relaxed
                ) % CLNUM;
  CLTBL[i]
}
// ----------------------------------------
// スライス表示対応
type T = f32;
pub struct SliceView {
  // スライスする立方体定義
  pub x:(T, T), pub y:(T, T), pub z: (T, T),
}
impl SliceView {
  pub fn view(&self, x:T, y:T, z:T)
    -> (T, T, T)
  {
    if x < self.x.0 || x > self.x.1 ||
       y < self.y.0 || y > self.y.1 ||
       z < self.z.0 || z > self.z.1 
    {
      (T::MAX, T::MAX, T::MAX) // 範囲外
    } else {
      (x, y, z)                // 範囲内
    }
  }
}
// ----------------------------------------
// 3Dviewオブジェクト
use wio_sbeye::WioSBEye;
use wio_sbcamera::WioSBCamera;
pub struct View3D {
  pub sb: WioSBCamera, // 球面束縛カメラ
  cr    : (T, T, T),   // 補正率
}
impl View3D {
  pub fn new (cr: (T, T, T)) -> Self {
    // カメラ作成
    let mut sb = WioSBCamera
                 ::build_default(240., 240.);
    sb.set_znear(1.0).set_zfar(1000.);
    Self {sb, cr}
  }
}
// 立体視変換
impl View3D {
  pub fn conv(&self, x:T, y:T, z:T) -> (T, T) 
  {
    let (x1, y1) = self.sb.convertf32((
       x*self.cr.0,
       y*self.cr.1, 
       z*self.cr.2,
    ));
    (x1/self.cr.0, y1/self.cr.1)
  }
}
pub fn wio_sbeye() -> WioSBEye {
  WioSBEye { // カメラ視点
    hw: 0,
    vw: 0,
    r : 700,
    r_limit: 1000,
    step: 15,
  }
}

    