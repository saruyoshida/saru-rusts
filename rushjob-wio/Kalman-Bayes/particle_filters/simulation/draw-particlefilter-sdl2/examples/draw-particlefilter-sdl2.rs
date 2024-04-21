pub use embedded_graphics::{
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

use sim_config::*;
use emb_bargraph::*;
use emb_shapegraph::*;
// ----------------------------------------
// グラフ表示設定
// 表示色設定
pub const BASE_COLOR : Rgb565 = Rgb565::BLACK;
pub const SCALE_COLOR: Rgb565 = Rgb565::WHITE;
pub const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
pub const RT_COLOR   : Rgb565 = Rgb565::RED;
pub const INIT_COLOR : Rgb565 = Rgb565::GREEN;
pub const PT_COLOR   : Rgb565 = Rgb565::CYAN;
pub const MU_COLOR   : Rgb565 =Rgb565::YELLOW;
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
                     "PrticleFilter", 
                     &output_settings
                   );
// ----------------------------------------
// グラフ表示設定
  // グラフ設定
  let mut eb = bargraph_setting();
  // 粒子、位置表示設定
  let mut es = EmbShapegraph::new(&eb); 
// ======================================
  // 画面クリア
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();
  // 目盛表示
  eb.mode_scale().draw(&mut display).unwrap();
  // ======================================
  // シミュレーション設定
  let mut sim = SimConfig::new();
  // ======================================
  // 粒子フィルタ設定
  let mut pf = sim.particlefilter();
  // ======================================
  // 粒子生成
  pf.create_particles(&sim.param);
  // 初期粒子描画
  es.set_shape_color(INIT_COLOR)
    .mode_fillcircle()
    .set_shape_diameter(1);
  pf.pt.iter().for_each(|pt|
    es.set_data(pt[0], pt[1])
      .draw(&mut display)
      .unwrap()
  );
  window.update(&display);

  // 制御入力作成
  let cmd = sim.make_cmd();
  // 繰返し観測
  for (i, u) in cmd.enumerate() {
    // シミュレータ位置更新
    sim.set_u(u).move_next();
    // 予測
    pf.set_u(u).predict();
    // 更新
    pf.update(sim.lms(), sim.zs());
    // 再サンプリング判定
    if pf.neff() < sim.jval {
      // 再サンプリング
      pf.resample();
    }
    // 平均、分散算出
    let (mu, _var) = pf.estimate();

    if i % sim.dstep == 0 {
      // 粒子描画
      es.set_shape_color(PT_COLOR)
        .mode_fillcircle()
        .set_shape_diameter(1);
      pf.pt.iter().for_each(|pt|
        es.set_data(pt[0], pt[1])
          .draw(&mut display)
          .unwrap()
      );
      // 実際位置描画
      es.set_shape_color(RT_COLOR)
        .mode_fillcircle()
        .set_shape_diameter(6)
        .set_data(sim.pos[0], sim.pos[1])
        .draw(&mut display)
        .unwrap();
      // 推定位置描画
      es.set_shape_color(MU_COLOR)
        .mode_filltriangle()
        .set_shape_diameter(6)
        .set_data(mu[0], mu[1])
        .draw(&mut display)
        .unwrap();

      window.update(&display);
    }
  }
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {
      break 'running Ok(());
    }
  }
}
// ========================================
// グラフセッティング
fn bargraph_setting() -> EmbBargraph<'static>
{
  let mut bargraph = EmbBargraph::new(
    DSP_ST,            // 表示開始位置
    DSP_SZ,            // 表示サイズ
    DSP_XRS..DSP_XRE,  // X目盛レンジ
    DSP_YRS..DSP_YRE,  // Y目盛レンジ
    DSP_HSP,           // 補正率(x,y)
    DSP_SN,            // 目盛刻み
                       // タイトル
    "pf sim",
  );
  bargraph.set_base_color(BASE_COLOR)
          .set_text_color(SCALE_COLOR)
          .set_scale_color(SCALE_COLOR)
          .set_box_color(BOX_COLOR);
  bargraph
}
