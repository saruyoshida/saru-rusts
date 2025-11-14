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

// キャビティ流れ
use cavityflow3d::*;
// グラフ供給
use graph_supply_quiver3d::*;

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
                     "cavityflow3d",
                     &output_settings
                   );
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, 
       mut graph_obj,
       mut graph_obj2) =
    graph_supply(
       0..200,       // x目盛
       0..200,       // y目盛
       (100., 100.), // 補正率
       (200, 200),   // 目盛刻み
       true,         // ｶﾗｰﾏｯﾌﾟ適用
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
    2., 2., 2., (100., 100., 100.)
  );
  eye.right().down().out();      // 初期設定
  p3.sb.set_eye(eye.position()); // 視点設定
  eye.step = 1;      // カメラ移動ステップ変更
  // 結果描画 -----------------------------
  let (nx, ny, nz) = (21, 21, 21);
  let cf3d = cavityflow3d(
    nx, ny, nz,      // グリッド数(x, y, z)
    20,              // 収束判定回数(圧力部分)
    40.,             // ﾚｲﾉﾙｽﾞ数
    0.01,            // Δt
    1./(nx-1) as T,  // Δx
    1./(ny-1) as T,  // Δy
    1./(nz-1) as T,  // Δz
    1e-5,            // 誤差閾値
  );
  for (u, v, w) in cf3d {
    gb.draw(&mut display).unwrap();// 画面ｸﾘｱ
    // 視点回転  回転しない場合以下ｺﾒﾝﾄｱｳﾄ
    eye.right();                   // 視点変更
    p3.sb.set_eye(eye.position()); // 視点設定
    // グラフ枠表示
    gl.reset_data();
    ruled_line().take(12)
    .for_each(|(line, color)| {
      let (x, y) = p3.conv(line);
      gl.set_data(x, y);
      gl.set_color(color);
      graph_supply_draw!(
        gl, display, Go1, Go2,
      );
    });
    // メッシュグリッド
    meshgrid(nx, ny, nz, eye.hw, 1)
    .for_each(|((k, y), (j, z), (i, x))| {
/*
      // 可変長矢印 始点終点取得
      let (p, norm) = make_argument3d(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        1.2,                    // 長さ調整
      );
*/
      // 固定長長矢印 始点終点取得
      let (p, norm) = make_argument3d_fixed(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        0.08,                   // 長さ調整
      );
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
  hw: i32, step:U      // カメラ角度, ステップ
)-> impl Iterator<Item=((U, T),(U, T),(U, T))>
{
  let (sx, sy, sz) = (0., 0., 0.);
  let (ex, ey, ez) = (2., 2., 2.);

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
