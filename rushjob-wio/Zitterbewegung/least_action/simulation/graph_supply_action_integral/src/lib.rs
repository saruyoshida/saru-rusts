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
pub fn graph_supply() -> (
         [EmbBargraph<'static>; GSC],
         [[EmbGraphs; SUBGSC] ; GSC],
)
{
  let gb  = [graph_box(0 , 5),
            ];
  let gos = [graph_obj(&gb[0]),
            ];

  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  x: i32, y: i32
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ  , x目盛   , y目盛
    (x, y)  , (315, 230), 100..300, -170..-90,
    //補正率    , 目盛刻み, 
    (100., 100.), (50, 20), 
    //ﾀｲﾄﾙ
    "action_integral",
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
          es.set_shape_color(CYAN)
            .set_shape_diameter(8)
            .mode_fillcircle();
  
  [EmbGraphs::Go1(es),
  ]
}
// ========================================
// 表示色設定 
const BLACK : Rgb565 = Rgb565::BLACK;
const CYAN  : Rgb565 = Rgb565::CYAN;

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


