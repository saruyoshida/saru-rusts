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
use micromath::F32Ext;

type T = f32;

// 電子の自由運動
use qms::*;
// グラフ供給
use graph_supply_qm::*;

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
       0..500,      // x目盛
       -500..200,   // y目盛
       (10., 100.), // 補正率
       (200, 100),   // 目盛刻み
    );
  let g = &mut graph_obj[0][0];
  // 目盛表示
  graph_box.iter_mut().for_each(|g| 
    g.mode_scale().draw(&mut display).unwrap()
  );
  // 結果描画 -----------------------------
  // シード値設定
  let seed: u8 = 10;
  // 平均前方微分DX(t)
  // ※重ね合わせ
  let d3 = move |xt: T, _| -xt.tan();
  g.reset_data();
  qms(d3, 0., 0.01, 5000, seed)
  .for_each(|(t, x)| {
     g.set_data(t, x); 
     // graph_supply_drawマクロによる実装
     graph_supply_draw!(g, display, Go1,)
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}
