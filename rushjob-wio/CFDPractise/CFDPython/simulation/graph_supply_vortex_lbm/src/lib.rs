#![no_std]
use emb_bargraph::*;
use emb_shapegraph::*;
// グラフ供給配列数
const GSC   : usize = 1;
// サブ配列数
const SUBGSC: usize = 1;
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
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> (
  [EmbBargraph<'static>; GSC],
  [EmbGraphs; SUBGSC],
)
{
  let gb  = [
    graph_box(
      0  , 2, xsr.clone(), ysr.clone(),
              (cr.0, cr.1), (sn.0, sn.1),
    ),
  ];
  let gos = [
    graph_obj(&gb[0]),
  ];
  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  x: i32, y: i32,
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ,x目盛,y目盛,補正率,目盛
    (x, y)  , (315,128), xsr, ysr, cr, sn,
    //ﾀｲﾄﾙ
    "vortex_lbm"
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
            .set_shape_width(2)
            .mode_fillrectangle();

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
/*
    Rgb565::CSS_RED,
    Rgb565::CSS_CRIMSON,
    Rgb565::CSS_DARK_RED,
*/
];
pub fn colormap(v: f32, min: f32, max: f32)
-> Rgb565 {
  // 正規化
  let i = (v - min) / (max - min);
  // index=v/閾値/(1/色数(16))の切捨て
  let i = (i / (1. / CLNUM as f32)) as usize;
  // index>=色数の場合、index=色数-1
  if i >= CLNUM {
    CLTBL[CLNUM-1]
  } else {
    CLTBL[i]
  }
}

