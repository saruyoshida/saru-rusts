#![no_std]
use emb_bargraph::*;
use emb_linegraph::*;
use emb_shapegraph::*;

// グラフ供給配列数
const GSC   : usize = 2;
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
  let gb  = [graph_box(0  , 50,  1),
             graph_box(148, 50,  2),

            ];
  let gos = [graph_obj(&gb[0]),
             graph_obj(&gb[1]),
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
    //開始  , 表示ｻｲｽﾞ, x目盛     , y目盛
    (x, y)  , (158, 158), 0..100, -10..40,
    //補正率  , 目盛刻み, 
    (10., 1.), (25, 10), 
    //ﾀｲﾄﾙ
    match title_no {
      1 => "cv normal",
      _ => "epsilon=4",
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

