#![no_std]
use emb_bargraph::*;
use emb_shapegraph::*;
use emb_arrowgraph::*;
// グラフ供給配列数
const GSC   : usize = 1;
// サブ配列数
//const SUBGSC: usize = 2;
// グラフ供給種類
type Go1 = EmbShapegraph;
type Go2 = EmbArrowgraph;
pub enum EmbGraphs {
  Go1(Go1), Go2(Go2),
}
impl EmbGraphs {
  // graph_supply_implマクロによる実装
  graph_supply_impl!(
    Go1, Go2,
  );
}
type T = f32;
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
  EmbGraphs,
  EmbGraphs,
)
{
  let gb  = [
    graph_box(
      0  , 0, xsr.clone(), ysr.clone(),
              (cr.0, cr.1), (sn.0, sn.1),
    ),
  ];
  let (gos1, gos2) = graph_obj(&gb[0]);

  (gb, gos1, gos2)
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
    (x, y)  , (315,252), xsr, ysr, cr, sn,
    //ﾀｲﾄﾙ
    "vortex_lbm_arrow"
  );
  gb.set_box_color(Rgb565::BLACK);
  gb
}
// ========================================
// グラフオブジェクト
fn graph_obj(bg: &EmbBargraph) 
-> (EmbGraphs, EmbGraphs) {
  // オブジェクト1
  let mut es = EmbShapegraph::new(bg); 
          es.set_shape_diameter(1)
            .set_shape_width(2)
            .mode_fillrectangle()
  ;
  // オブジェクト2 渦度ベクトル
  let mut ea = EmbArrowgraph::new(bg, true);
          ea.mode_fillarrow()
            .set_draw_th((-0.1, 0.1))
            .set_color_th((-0.1, 0.1));

  (EmbGraphs::Go1(es),
   EmbGraphs::Go2(ea),
  )
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
  // 矢印データセット
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
// 引数作成:固定長
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
//===========================================
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
//===========================================
// メソッド調整
pub trait SDATrait {
  fn set_data_arrow(
    &mut self, 
    _data : ((T, T), (T, T)),
    _norm : T,
  ) -> &mut Self {self}
}
impl SDATrait for EmbShapegraph{}


