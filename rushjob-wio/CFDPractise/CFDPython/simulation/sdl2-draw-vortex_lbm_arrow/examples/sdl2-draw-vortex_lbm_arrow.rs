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
use itertools::{iproduct};
use linspacef32::linspacef32;

// 格子ボルツマン渦
use vortex_lbm::*;
// グラフ供給
use graph_supply_vortex_lbm_arrow::*;

type T = f32;
type U = usize;
// ----------------------------------------
fn main() -> 
  Result<(), core::convert::Infallible>
{
  let mut display: SimulatorDisplay<Rgb565> 
    = SimulatorDisplay::new(
                          Size::new(320, 240)
                        );
  let output_settings = OutputSettingsBuilder
                        ::new().scale(2)
                               .build();
  let mut window = Window::new(
                     "vortex_lbm", 
                     &output_settings
                   );
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, mut goj1, mut goj2) =
    graph_supply(
       200..1800,    // x目盛
       -400..1200,   // y目盛
       (10., 10.),   // 補正率
       (1800, 1600), // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g  = &mut goj1;
  let ga = &mut goj2;
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 3d化     -----------------------------
  let (mut p3, mut eye) = Plot3D::new(
    200., 80., 80., (10., 10., 10.)
  );
  eye.step = 80;     // カメラ移動ステップ変更
  eye.down();
  p3.sb.set_eye(eye.position()); // 視点設定
  eye.step = 1;      // カメラ移動ステップ変更
  let mut rcnt = 0;  // 回転頻度用変数
  // 結果描画 -----------------------------
  let (ni, nj) = (80, 200); // 行列数
  let vl = vortex_lbm(
    ni,    // 格子行
    nj,    // 格子列
    0.02,  // 液体粘度
    0.1,   // 初速と流入速度 
  )
  .skip(20000)
  .step_by(20);
  for u in vl {
    gb.draw(&mut display).unwrap();

    // 視点回転 -----------------------------
    rcnt += 1;
    if rcnt > 0 { // 回転スピード調整
      eye.right();                   // 変更
      p3.sb.set_eye(eye.position()); // 設定
      rcnt = 0;
    }

    // 下ベクトル表示 -----------------------
    meshgrid(ni-1, nj-1, eye.hw, 2)
    .filter(|((i, _), (j, _))| 
      u[(*i,*j)] < -0.02 && *j != 0
    )
    .for_each(|((i, y), (j, x))| {
      // 可変長矢印 始点終点取得
      let (p, _) = make_argument3d(
        (x, y, 0.),             // grid位置                       
        (0., 0., u[(i,j)]),     // ﾍﾞｸﾄﾙ
        1000.,                  // 長さ調整
      );
      // 始点、終点立体視化
      let (start, end) = (
        p3.conv(p.0), p3.conv(p.1)
      );
      // 矢印グラフ描画
      ga.set_data_arrow((start,end),u[(i,j)]);
      graph_supply_draw!(
        ga, display, Go1, Go2,
      );
    });
    // lbm平面描画 -------------------------
    meshgrid(ni-1, nj-1, eye.hw, 1)
    .filter(|((i, _), (j, _))|
/*
    // 渦だけ表示したい場合
      (u[(*i, *j)] >  0.02 ||
       u[(*i, *j)] < -0.02) &&
*/
      *j != 0
    )
    .for_each(|((i, y), (j, x))| {
      let (xp, yp) = p3.conv((x, y, 0.));
      g.set_data(xp, yp);
      g.set_color(
        colormap(u[(i, j)], -0.1, 0.1)
      );
      graph_supply_draw!(
        g, display, Go1, Go2,
      );
    });
    // 上ベクトル表示 -----------------------
    meshgrid(ni-1, nj-1, eye.hw, 2)
    .filter(|((i, _), (j, _))| 
      u[(*i,*j)] > 0.02 && *j != 0
    )
    .for_each(|((i, y), (j, x))| {
      // 可変長矢印 始点終点取得
      let (p, _) = make_argument3d(
        (x, y, 0.),             // grid位置                       
        (0., 0., u[(i,j)]),     // ﾍﾞｸﾄﾙ
        1000.,                  // 長さ調整
      );
      // 始点、終点立体視化
      let (start, end) = (
        p3.conv(p.0), p3.conv(p.1)
      );
      // 矢印グラフ描画
      ga.set_data_arrow((start,end),u[(i,j)]);
      graph_supply_draw!(
        ga, display, Go1, Go2,
      );
    });
    window.update(&display);
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {break;}
  }
  Ok(())
  // -------------------------------------
}
// 手抜きな描画順設定
use auto_enums::auto_enum;
#[auto_enum(Iterator)]
fn meshgrid(
  ni: U, nj: U,
  hw: i32, step:U   // カメラ角度, ステップ
)-> impl Iterator<Item=((U, T),(U, T))> {
  let (si, sj) = (0. , 0.  );
  let (ei, ej) = (79., 199.);

  if (0..90).contains(&hw) {
    iproduct!(
      linspacef32(si,ei,ni).enumerate()
                           .step_by(step),
      linspacef32(sj,ej,nj).enumerate()
                           .step_by(step),
    )
  } else if (90..180).contains(&hw) {
    iproduct!(
      linspacef32(si,ei,ni).enumerate().rev()
                           .step_by(step),
      linspacef32(sj,ej,nj).enumerate()
                           .step_by(step),
    )
  } else if (180..270).contains(&hw) {
    iproduct!(
      linspacef32(si,ei,ni).enumerate().rev()
                           .step_by(step),
      linspacef32(sj,ej,nj).enumerate().rev()
                           .step_by(step),
    )
  } else {
    iproduct!(
      linspacef32(si,ei,ni).enumerate()
                           .step_by(step),
      linspacef32(sj,ej,nj).enumerate().rev()
                           .step_by(step),
    )
  }
}
