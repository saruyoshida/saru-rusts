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
use vortex_lbm2dmix::*;
// グラフ供給
use graph_supply_vortex_lbm::*;

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
                     "vortex_lbm2dmix", 
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
       0..2000,    // x目盛
       0..800,     // y目盛
       (10., 10.), // 補正率
       (200, 200), // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  let (ni, nj) = (80, 200); // 行列数
  // 障壁
  let barrier = ((ni/2)-8..(ni/2)+8)
                .map(|i| (i, ni/2))
  ;
  // 格子ボルツマン法
  let vl = vortex_lbm2dmix(
    ni,     // 格子行
    nj,     // 格子列
    0.02,   // 動粘性係数
    0.1,    // 初速と流入速度 
    barrier // 障壁
  )
  .step_by(20);
  for u in vl {
    gb.draw(&mut display).unwrap();
    // meshgrid(ij)
    iproduct!(
      linspacef32(0., 79. , ni-1).enumerate(),
      linspacef32(0., 199., nj-1).enumerate(),
    )
/*
    // 渦だけ表示したい場合
    .filter(|((i, _), (j, _))|
      u[(*i, *j)] >  0.02 ||
      u[(*i, *j)] < -0.02
    )
*/
    .for_each(|((i, y), (j, x))| {
      g.set_data(x, y);        // grid位置
      g.set_color(
        colormap(u[(i, j)], -0.1, 0.1)
      );
      graph_supply_draw!(g, display, Go1,);
    });
    window.update(&display);
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {break;}
  }
  Ok(())
  // -------------------------------------
}

