#![no_std]
use emb_bargraph::*;
use emb_arrowgraph::*;
#[allow(unused_imports)]
use num_traits::Float;

type T = f32;
// グラフ供給配列数
const GSC   : usize = 1;
// サブ配列数
const SUBGSC: usize = 1;
// グラフ供給種類
type Go1 = EmbArrowgraph;
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
  cm  : bool,
) -> (
  [EmbBargraph<'static>; GSC],
  [EmbGraphs; SUBGSC],
)
{
  let gb  = [
    graph_box( 
      0  , 5,   1, xsr.clone(), ysr.clone(),
                  (cr.0, cr.1), (sn.0, sn.1),
    ),
  ];
  let gos = [
    graph_obj(&gb[0], cm),
  ];
  (gb, gos)
}
// ========================================
// グラフボックス
fn graph_box(
  x: i32, y: i32, _title_no: usize,
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (f32, f32),
  sn  : (usize, usize),
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始  , 表示ｻｲｽﾞ,x目盛,y目盛,補正率,目盛
    (x, y)  , (315, 233), xsr, ysr, cr, sn,
    //ﾀｲﾄﾙ
    "quiver"
  );
  gb.set_box_color(Rgb565::BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(bg: &EmbBargraph, cm: bool) 
-> EmbGraphs
{
  // オブジェクト1
  let mut es = EmbArrowgraph::new(bg, cm);
  es.mode_fillarrow();

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
  pub fn set_data(&mut self, x: T, y: T) {
    match self {
      $(Self::$go(f) => {f.set_data(x, y);},)*
  }}
  pub fn set_data_arrow(
    &mut self, data:((T, T), (T, T)), norm: T
  ) {
    match self {
      $(Self::$go(f) => 
          {f.set_data_arrow(data, norm);},)*
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
// 引数作成:可変長
#[allow(clippy::type_complexity)]
pub fn make_argument2d(
  xy  : (T, T),
  uv  : (T, T),
  base: T
) -> (((T, T), (T, T)), T) {
  EmbArrowgraph::make_argument2d(xy, uv, base)
}
//　引数作成:固定長
#[allow(clippy::type_complexity)]
pub fn make_argument2d_fixed(
  xy  : (T, T),
  uv  : (T, T),
  base: T
) -> (((T, T), (T, T)), T) {
  EmbArrowgraph::make_argument2d_fixed(
    xy, uv, base
  )
}