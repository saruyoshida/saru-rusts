pub use embedded_graphics::{
  pixelcolor::Rgb565,
  primitives::{Rectangle,PrimitiveStyle},
  prelude::*,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, 
    SimulatorDisplay, 
    SimulatorEvent, 
    Window,
};
pub use std::{thread, time::Duration};

// use num_traits::Float;
#[allow(clippy::single_component_path_imports)]
#[allow(unused_imports)]
use auto_allocator;
use itertools::{iproduct};
use linspacef32::linspacef32;

// 格子ボルツマン法3D
use vortex_lbm3d::*;
// グラフ供給
use graph_supply_vortex_lbm3d::*;

type T = f32;
type U = usize;
// ----------------------------------------
fn main() -> 
  Result<(), core::convert::Infallible>
{
  // 画面サイズ
  let xsz = (0_i32, 470);
  let ysz = (0_i32, 156);
  // シミュレータディスプレイ
  let mut display: SimulatorDisplay<Rgb565> =
    SimulatorDisplay::new(
      Size::new(xsz.1, ysz.1)
    );
  let output_settings = 
    OutputSettingsBuilder::new().scale(2)
                                .build();
  let mut window = Window::new(
                     "vortex_lbm3d",
                     &output_settings
  );
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(xsz.0, ysz.0), 
    Size::new(xsz.1, ysz.1)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, 
       mut graph_obj,
       mut graph_obj2) =
    graph_supply(
       xsz, ysz,
       0..750,       // x目盛
       0..250,       // y目盛
       (10., 10.),   // 補正率
       (750, 250),   // 目盛刻み
       true,         // ｶﾗｰﾏｯﾌﾟ適用
       (0., 0.1),    // 色閾値
       (0.0001, 20.) // 描画閾値①
    );
  let gb = &mut graph_box[0];
  let ga = &mut graph_obj;
  let gl = &mut graph_obj2;
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 3d化     -----------------------------
  let (mut p3, mut eye) = Plot3D::new(
    75., 25., 25., (10., 10., 10.)
  );
  eye.left().left().down().out();// 初期設定
  p3.sb.set_eye(eye.position()); // 視点設定
  eye.step = 1; // カメラ移動ステップ変更
  // 結果描画 -----------------------------
  // グリッド数
  let (nx, ny, nz) = (75, 25, 25);
/*
  // 円柱障壁を設定
  let obr = 3.;           // 円柱半径
  let oc  = ny as T / 2.; // 中心点
  let barrier = 
    iproduct!(0..ny, 0..nx, 0..nz)
    .map(|(y, x, z)| (
      (y, x, z),
      ((y as T - oc).powi(2) +
       (x as T - oc).powi(2)
      ).sqrt(),
    ))
    .filter(|(_,pos)|*pos<=obr && *pos>obr-1.)
    .map(|(yxz, _)| yxz)
  ;
*/
  // 板障壁を設定
  let barrier = 
    iproduct!((ny/2)-2..(ny/2)+2, 0..nz)
    .map(|(y, z)| (y, ny/2, z))
  ;

  // 格子ボルツマン法
  let lbm3d = vortex_lbm3d(
    ny, nx, nz, // グリッド数(y, x, z)
    0.008,      // 動粘性係数
    0.1,        // 初速と流入速度 
    barrier,    // 障壁
  )
  //.skip(1000)
  .step_by(5)
  ;
  for (v, u, w) in lbm3d {
    gb.draw(&mut display).unwrap();// 画面ｸﾘｱ
    // 視点回転  回転しない場合以下ｺﾒﾝﾄｱｳﾄ
  //eye.right();                   // 視点変更
  //p3.sb.set_eye(eye.position()); // 視点設定
    // グラフ枠表示
    gl.reset_data();
    ruled_line((0., 75.),(0., 25.),(0., 25.))
    .take(12)
    .for_each(|(line, color)| {
      let (x, y) = p3.conv(line);
      gl.set_data(x, y);
      gl.set_color(color);
      graph_supply_draw!(
        gl, display, Go1, Go2,
      );
    });
    // メッシュグリッド
  //meshgrid(nx-2, ny-2, nz-2, eye.hw, 2) //②
    meshgrid(nx-2, ny-2, nz-2, eye.hw, 1) //①
    .filter(|((j, _), (i, _), (k, _))| 
      *j != 0 && *i != 0 && *k != 0 
    )
    .map(|((j, y), (i, z), (k, x))|
/*
      // 可変長矢印 始点終点取得 ②
      make_argument3d(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        120.,                    // 長さ調整
      )
*/
      // 固定長長矢印 始点終点取得 ①
      make_argument3d_fixed(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        0.08,                   // 長さ調整
      )

    )
  //.filter(|(_, norm)| *norm > 0.02) //②
    .filter(|(_, norm)| *norm > 0.01) //①
    .for_each(|(p, norm)| {
      // 始点、終点立体視化
      let (start, end) = (
        p3.conv(p.0), p3.conv(p.1)
      );
      // 矢印グラフ描画
      ga.set_data_arrow((start, end), norm);
      graph_supply_draw!(
        ga, display, Go1, Go2,
      );
    });
    window.update(&display);
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {return Ok(())}
  }
  // -------------------------------------
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {break 'running Ok(());}
  }
}
// 手抜きな描画順設定
use auto_enums::auto_enum;
#[auto_enum(Iterator)]
fn meshgrid(
  nx: U, ny: U, nz: U, // x, y, z
  hw: i32, step: U     // カメラ角度, ステップ
)-> impl Iterator<Item=((U, T),(U, T),(U, T))>
{
  let (sx, sy, sz) = (0., 0., 0.);
  let (ex, ey, ez) = (74., 24., 24.);

  if (0..90).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate().rev()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate()
                           .step_by(step),
    )
  } else if (90..180).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate()
                           .step_by(step),
    )
  } else if (180..270).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate().rev()
                           .step_by(step),
    )
  } else {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate().rev()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate().rev()
                           .step_by(step),
    )
  }
}


