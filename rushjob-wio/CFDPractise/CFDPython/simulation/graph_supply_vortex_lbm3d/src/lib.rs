#![no_std]
use emb_bargraph::*;
use emb_linegraph::*;
use emb_arrowgraph::*;
#[allow(unused_imports)]
use num_traits::Float;

type T = f32;
// グラフ供給配列数
const GSC   : usize = 1;
// サブ配列数
//const SUBGSC: usize = 2;
// グラフ供給種類
type Go1 = EmbArrowgraph;
type Go2 = EmbLinegraph;
pub enum EmbGraphs {
  Go1(Go1), 
  Go2(Go2), 
}
impl EmbGraphs {
  // graph_supply_implマクロによる実装
  graph_supply_impl!(
    Go1, Go2,
  );
}
// ----------------------------------------
// グラフ供給格納
#[allow(clippy::too_many_arguments)]
pub fn graph_supply(
  x   : (i32, u32),
  y   : (i32, u32),
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (T, T),
  sn  : (usize, usize),
  cm  : bool,
  color_th: (T, T),
  draw_th : (T, T),
) -> (
  [EmbBargraph<'static>; GSC],
  EmbGraphs,
  EmbGraphs,
)
{
  let gb  = [
    graph_box( 
      x, y, 1, xsr.clone(), ysr.clone(),
               cr, sn
    ),
  ];
  let gos  = graph_obj(
    &gb[0], cm, color_th, draw_th
  );
  let gos2 = graph_obj2(&gb[0]);

  (gb, gos, gos2)
}
// ========================================
// グラフボックス
fn graph_box(
  x   : (i32, u32), 
  y   : (i32, u32),
  _title_no: usize,
  xsr : Range<i32>,
  ysr : Range<i32>,
  cr  : (T, T),
  sn  : (usize, usize),
) -> EmbBargraph<'static>  
{
  // ボックス1
  let mut gb = EmbBargraph::new(
    //開始, 表示ｻｲｽﾞ,x目盛,y目盛,補正率,目盛
    (x.0, y.0), (x.1, y.1), xsr, ysr, cr, sn,
    //ﾀｲﾄﾙ
    "vortex_lbm3d"
  );
  gb.set_box_color(Rgb565::BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(
  bg: &EmbBargraph, 
  cm: bool,
  color_th: (T, T),
  draw_th : (T, T),
) -> EmbGraphs {
  // オブジェクト1
  let mut ea = EmbArrowgraph::new(bg, cm);
          ea.mode_fillarrow()
            .set_color_th(color_th)
            .set_draw_th(draw_th);    // 閾値

  EmbGraphs::Go1(ea)
}
fn graph_obj2(bg: &EmbBargraph) -> EmbGraphs
{ // オブジェクト2
  let el = EmbLinegraph::new(bg);
  EmbGraphs::Go2(el)
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
  // リセットデータ
  pub fn reset_data(&mut self) {
    match self {
      $(Self::$go(f) => {
          f.reset_data();
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
//===========================================
// 引数作成:可変長
#[allow(clippy::type_complexity)]
pub fn make_argument3d(
  xyz : (T, T, T),
  uvw : (T, T, T),
  base: T
) -> (((T, T, T), (T, T, T)), T) {
  EmbArrowgraph::make_argument3d(
    xyz, uvw, base
  )
}
//　引数作成:固定長
#[allow(clippy::type_complexity)]
pub fn make_argument3d_fixed(
  xyz : (T, T, T),
  uvw : (T, T, T),
  base: T
) -> (((T, T, T), (T, T, T)), T) {
  EmbArrowgraph::make_argument3d_fixed(
    xyz, uvw, base
  )
}
//===========================================
// グラフ立体視化オブジェクト
use wio_sbeye::WioSBEye;
use wio_sbcamera_ortho::WioSBCameraOrtho;
pub struct Plot3D {
  pub sb : WioSBCameraOrtho,
  cr     : (T, T, T),
}
impl Plot3D {
  pub fn new (
    w: T, h: T, d: T, cr: (T, T, T)
  ) -> (Self, WioSBEye) {
    // カメラ作成
    let mut sb = WioSBCameraOrtho::new(
      w*cr.0, h*cr.1, d*cr.2
    );
    let eye = WioSBEye { // カメラ視点
      hw     : 0,
      vw     : 0,
      r      : 100,
      r_limit: 1000,
      step   : 10,
    };
    sb.set_eye(eye.position()); // 視点設定
    (Self {sb, cr}, eye)
  }
}
// 立体視変換
impl Plot3D {
  pub fn conv(&self, xyz:(T, T, T)) -> (T, T)     
  {
    let xyz3d = self.sb.convertf32(
      (xyz.0*self.cr.0, 
       xyz.1*self.cr.1,
       xyz.2*self.cr.2,
      )
    );

    (xyz3d.0/self.cr.0, xyz3d.1/self.cr.1)
  }
}
pub fn ruled_line(
  x: (T, T),
  y: (T, T),
  z: (T, T),
)
-> impl Iterator<Item=((T, T, T), Rgb565)> {
  let scale =         // グラフ枠一筆描き
    [(x.1, y.0, z.1), // 右奥下
     (x.1, y.0, z.0), // 右前下
     (x.0, y.0, z.0), // 左前下
     (x.0, y.1, z.0), // 左前上
     (x.0, y.1, z.1), // 左奥上
     (x.0, y.0, z.1), // 左奥下
     (x.1, y.0, z.1), // 右奥下
     (x.1, y.1, z.1), // 右奥上
     (x.0, y.1, z.1), // 左奥上
     (x.0, y.0, z.1), // 左奥下
     (x.0, y.0, z.0), // 左前下
     (x.0, y.1, z.0), // 左前上
     (x.1, y.1, z.0), // 右前上
     (x.1, y.1, z.1), // 右奥上
    ]
  ;
  let scolr =                // グラフ枠色
    [Rgb565::CSS_LIGHT_GRAY, // 
     Rgb565::CSS_LIGHT_GRAY, // 
     Rgb565::WHITE,          // 
     Rgb565::WHITE,          // 
     Rgb565::CSS_LIGHT_GRAY, // 
     Rgb565::CSS_GRAY,       // 
     Rgb565::CSS_GRAY,       // 
     Rgb565::CSS_GRAY,       // 
     Rgb565::CSS_GRAY,       // 
     Rgb565::CSS_GRAY,       // 
     Rgb565::CSS_LIGHT_GRAY, // 
     Rgb565::WHITE,          // 
     Rgb565::WHITE,          // 
     Rgb565::CSS_LIGHT_GRAY, // 
    ]
  ;
  scale.into_iter().zip(scolr)
}
// メソッド調整
pub trait SDATrait {
  fn set_data_arrow(
    &mut self, 
    _data : ((T, T), (T, T)),
    _norm : T,
  ) -> &mut Self {self}
}
impl SDATrait for EmbLinegraph{}

