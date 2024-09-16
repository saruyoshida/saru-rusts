#![no_std]
use emb_bargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;

// グラフ供給配列数
const GSC   : usize = 2;
// サブ配列数
const SUBGSC: usize = 3;
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
         [EmbGraphs; SUBGSC],
)
{
  let gb  = [graph_box1(0  , 30),
             graph_box2(148, 30),
            ];
  let gos = graph_obj(&gb[0], &gb[1]);

  (gb, gos)
}
// ========================================
// グラフボックス1
fn graph_box1(
  x: i32, y: i32
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ, x目盛     , y目盛
    (x, y)  , (158, 158), 0..150, -20..100,
    //補正率  , 目盛刻み, //ﾀｲﾄﾙ
    (10., 1.), (50, 20), "z, filter"
  );
  gb.set_box_color(BLACK);
  gb
}
// グラフボックス2
fn graph_box2(
  x: i32, y: i32
) -> EmbBargraph<'static>  
{
  // ボックス2
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ, x目盛     , y目盛
    (x, y)  , (158, 158), 0..150, -20..100,
    //補正率  , 目盛刻み, //ﾀｲﾄﾙ
    (10., 10.), (50, 20), "residuals"
  );
  gb.set_box_color(BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(
  bg : &EmbBargraph,
  bg2: &EmbBargraph,
) -> [EmbGraphs; SUBGSC]
{
  // オブジェクト1
  let mut es = EmbShapegraph::new(bg); 
          es.set_shape_color(RED)
            .set_shape_diameter(1)
            .mode_fillcircle();
  // オブジェクト2
  let mut el1 = EmbLinegraph::new(bg);
          el1.set_shape_color(CYAN);
  // オブジェクト3
  let mut el2 = EmbLinegraph::new(bg2);
          el2.set_shape_color(YELLOW);
  
  [EmbGraphs::Go1(es),
   EmbGraphs::Go2(el1),
   EmbGraphs::Go2(el2),
  ]
}
// ========================================
// 表示色設定 
const BLACK : Rgb565 = Rgb565::BLACK;
const CYAN  : Rgb565 = Rgb565::CYAN;
const RED   : Rgb565 = Rgb565::RED;
const YELLOW: Rgb565 = Rgb565::YELLOW;

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


