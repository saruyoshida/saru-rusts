#![no_std]
use emb_bargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;

// グラフ供給配列数
const GSC   : usize = 4;
// サブ配列数
const SUBGSC: usize = 2;
// グラフ供給種類
type Go1 = EmbShapegraph;
type Go2 = EmbLinegraph;
pub enum EmbGraphs {
  Go1(Go1), 
  Go2(Go2),
}
impl EmbGraphs {
  // graph_supply_implマクロによる実装
  graph_supply_impl!(
    Go1,
    Go2,
  );
}
// ----------------------------------------
// グラフ供給格納
pub fn graph_supply() -> (
         [EmbBargraph<'static>; GSC],
         [[EmbGraphs; SUBGSC] ; GSC],
)
{
  let gb  = [graph_box(0  , 2,    1),
             graph_box(158, 2,    2),
             graph_box(0  , 118,  3),
             graph_box(158, 118,  4),
            ];
  let gos = [graph_obj(&gb[0]),
             graph_obj(&gb[1]),
             graph_obj(&gb[2]),
             graph_obj(&gb[3]),
            ];

  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  x: i32, y: i32, title_no: usize 
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
/*
    //開始  , 表示ｻｲｽﾞ, x目盛     , y目盛
    (x, y)  , (158, 108), -25..175, -20..120,
    //補正率  , 目盛刻み, 
    (10., 1.), (25, 20),
*/
    //開始  , 表示ｻｲｽﾞ, x目盛     , y目盛
    (x, y)  , (158, 108), -25..175, -20..60,
    //補正率  , 目盛刻み, 
    (10., 1.), (25, 20), 
    //ﾀｲﾄﾙ
    match title_no {
      1 => "cv normal",
      2 => "cv Q(var=2.0)",
      3 => "cv Q(var=50.0)",
      _ => "ca normal",
    }
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
  let mut es = EmbShapegraph::new(bg); 
          es.set_shape_color(RED)
            .set_shape_diameter(1)
            .mode_fillcircle();
  // オブジェクト2
  let mut el1 = EmbLinegraph::new(bg);
          el1.set_shape_color(CYAN);
  
  [EmbGraphs::Go1(es),
   EmbGraphs::Go2(el1),
  ]
}
// ========================================
// 表示色設定 
const BLACK : Rgb565 = Rgb565::BLACK;
const WHITE : Rgb565 = Rgb565::WHITE;
const CYAN  : Rgb565 = Rgb565::CYAN;
const RED   : Rgb565 = Rgb565::RED;

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


