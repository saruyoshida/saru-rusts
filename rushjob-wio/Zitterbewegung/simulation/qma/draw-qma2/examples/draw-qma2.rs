#![no_std]
#![no_main]
// wio_terminal 0.7.2対応 ---
use wio_terminal as wio;
use wio::entry;
use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
// --------------------------
use panic_halt as _;
use embedded_graphics::{
  primitives::{Rectangle,PrimitiveStyle},
  pixelcolor::Rgb565,
  prelude::*,
};
#[allow(unused_imports)]
use num_traits::Float;

// 確率微分方程式
use qma::*;
// グラフ供給
use graph_supply_qma::*;

type T = f32;
#[entry]
fn main() -> ! {
  // wio_terminal設定 0.7.2対応-----------
  let mut peripherals =
    Peripherals::take().unwrap();
  let core = CorePeripherals::take().unwrap();

  let mut clocks = GenericClockController
  ::with_external_32kosc(
    peripherals.GCLK,
    &mut peripherals.MCLK,
    &mut peripherals.OSC32KCTRL,
    &mut peripherals.OSCCTRL,
    &mut peripherals.NVMCTRL,
  );
  let mut delay = Delay::new(
    core.SYST, &mut clocks
  );
  let sets = wio::Pins::new(
    peripherals.PORT
  ).split();

  let (mut display, _backlight) = sets
    .display
    .init(
       &mut clocks,
       peripherals.SERCOM7,
       &mut peripherals.MCLK,
       58.MHz(),
       &mut delay,
    )
    .unwrap();
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, mut graph_obj) =
    graph_supply(
       -1200..1200,          // x目盛
       -1200..1200,          // y目盛
       -1200..1200,          // z目盛
       (100., 100. ,100.),   // 補正率
       (1200, 1200, 1200),   // 目盛刻み
       -1200..1200,          // ｽﾗｲｽ横目盛
       -1200..1200,          // ｽﾗｲｽ縦目盛
       (100., 100.),         // ｽﾗｲｽ補正率
       (1200, 1200),         // ｽﾗｲｽ目盛刻み
    );
  // スライス表示設定
  let sv = SliceView {
    x:(-10.,10.), y:(-10.,10.), z:(-1.,1.)
  };
  // 目盛表示
  graph_box.iter_mut().for_each(|g| 
    g.mode_scale().draw(&mut display).unwrap()
  );
  // 結果描画 -----------------------------
  // 水素原子
  // QMA2 第1励起(200軌道)
  let dxyz = |xt:T, yt:T, zt:T, rt:T| {
    let p = -((4.-rt)/(2.*rt*(2.-rt)));
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt)
  };
  // 見本経路描画                         
  let xyz0 = (1., 1., 1.); // 開始位置(x,y,z)
  let dt   = 0.01;         // Δt
  let cnt  = 300000;       // 計算回数
  let step = 10;           // 描画間引き数

  (0..1u8)                 // 電子数
  .map(|e| (120+86*e)%255) // 乱数シード値
  .flat_map(|seed| qma(dxyz,xyz0,dt,cnt,seed))
  .step_by(step)
  .enumerate()
  .for_each(|(i, (dx, dy, dz, _dr))| {
    graph_obj[0].set_data(dx, dy); // x-y平面
    graph_obj[1].set_data(dz, dy); // z-y平面
    graph_obj[2].set_data(dx, dz); // x-z平面
    // スライス表示
    let (sx, sy, _sz) = sv.view(dx, dy, dz);
    graph_obj[3].set_data(sx, sy);
    // 描画
    graph_obj.iter_mut().for_each(|g| {
      graph_supply_draw!(g, display, Go1,);
    });
    // 定期的に色を変える
    if i%(10000/step) == 0 {
      let scolor = color_change();
      graph_obj.iter_mut().for_each(|g| {
        g.set_color(scolor);
      })
    }
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}

