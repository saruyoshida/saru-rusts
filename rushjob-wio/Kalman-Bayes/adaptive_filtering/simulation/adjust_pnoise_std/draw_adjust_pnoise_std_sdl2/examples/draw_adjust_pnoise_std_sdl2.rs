use embedded_graphics::{
  primitives::{
    Rectangle,
    PrimitiveStyle,
  },
  pixelcolor::Rgb565,
  prelude::*,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, 
    SimulatorDisplay, 
    SimulatorEvent, 
    Window,
};
pub use std::{thread, time::Duration};
// フィルタ供給
use filter_supply_adjust_pnoise_std::*;
// グラフ供給
use graph_supply_adjust_pnoise_std::*;
// シミュレーションデータ
use generate_data::GenerateData;

fn main() -> 
  Result<(), core::convert::Infallible> {
  let mut display: SimulatorDisplay<Rgb565> 
    = SimulatorDisplay::new(
                          Size::new(320, 240)
                        );
  let output_settings = OutputSettingsBuilder
                        ::new().scale(2)
                               .build();
  let mut window = Window::new(
                     "", 
                     &output_settings
                   );
  // フィルタ/グラフ供給 -----------------
  let mut filters = filter_supply();
  let (mut graph_box, mut g) = graph_supply();
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  )
  .draw(&mut display)
  .unwrap();
  // 目盛表示
  graph_box.iter_mut().for_each(|gb| 
    gb.mode_scale().draw(&mut display)
                   .unwrap()
  );
  window.update(&display);
  // 繰り返し -----------------------------
  let dt = 0.1;
  let gd = GenerateData::new(150, 0.2);
  for f in filters.iter_mut() {
    let gdc = gd.clone();
    for (i, (pos, z)) in gdc.into_iter()
                            .enumerate() {
      // フィルタ操作
      f.z_set(0, z.0);  // 観測値設定
      f.predict();      // 予測
      f.update();       // 更新
      // グラフ描画
      let x = i as f32 * dt;
      g[0].set_data(x, pos.0); 
      g[1].set_data(x, f.x(0)); 
      g[2].set_data(x, f.x(1)); 
      g.iter().for_each(|go| 
        // graph_supply_drawマクロによる実装
        graph_supply_draw!(
          go, display, Go1, Go2,
        )
      );
      window.update(&display);
    }
    // グラフ描画エリアクリア
    graph_box.iter_mut().for_each(|gb| 
      gb.mode_clear().draw(&mut display)
                     .unwrap()
    );
    g.iter_mut().for_each(|go| 
      go.reset_data()
    );
    window.update(&display);
  }
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {
      break 'running Ok(());
    }
  }
}



