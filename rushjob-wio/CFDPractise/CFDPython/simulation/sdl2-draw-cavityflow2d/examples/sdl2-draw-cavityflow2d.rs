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

use itertools::{iproduct};
use linspacef32::linspacef32;

// キャビティ流れ
use cavityflow2d::*;
// グラフ供給
use graph_supply_quiver2d::*;

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
                     "cavityflow2d", 
                     &output_settings
                   );
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, mut graph_obj) =
    graph_supply(
       -10..210,     // x目盛
       -10..210,     // y目盛
       (100., 100.), // 補正率
       (50, 50),     // 目盛刻み
       true,         // ｶﾗｰﾏｯﾌﾟ適用
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  let (nx, ny) = (41, 41); // グリッド数
  let (lx, ly) = (2., 2.); // x,y長さ
  let cf2d = cavityflow2d(
    nx, ny,           // グリッド数(x, y)
    50,               // 計算回数(圧力部分)
    1., 0.1,          // 密度, 粘性係数
    0.001,            // Δt
    lx/(nx-1) as f32, // Δx
    ly/(ny-1) as f32, // Δy
  )
  .step_by(10);
  for (u, v) in cf2d {
    gb.draw(&mut display).unwrap();
    // meshgrid(ji)
    iproduct!(
      linspacef32(0., lx, nx).enumerate()
                             .step_by(2),
      linspacef32(0., ly, ny).enumerate()
                               .step_by(2)
    )
    .for_each(|((j, x), (i, y))| {
      // 可変長矢印
      let (a, norm) = make_argument2d(
        (x, y),                 // grid位置
        (u[(i, j)], v[(i, j)]), // ﾍﾞｸﾄﾙ
        1.2,                    // 長さ調整
      );
/*
      // 固定長矢印
      let (a, norm) = make_argument2d_fixed(
        (x, y),                 // grid位置
        (u[(i, j)], v[(i, j)]), // ﾍﾞｸﾄﾙ
        0.08,                   // 長さ調整
      );
*/
      g.set_data_arrow(a, norm);
      graph_supply_draw!(g, display, Go1,);
    });
    thread::sleep(Duration::from_millis(300));
    window.update(&display);
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {return Ok(())}
  }
  // -------------------------------------
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {
      break 'running Ok(());
    }
  }
}
