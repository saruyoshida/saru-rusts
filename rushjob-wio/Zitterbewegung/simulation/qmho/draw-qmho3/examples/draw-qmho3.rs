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
use num_traits::Float;

// 電子の自由運動
use qms::*;
// グラフ供給
use graph_supply_qm::*;

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
       0..1500,     // x目盛
       -100..100,   // y目盛
       (100., 10.), // 補正率
       (300, 20),   // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0][0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  let mut seed: u8 = 120;  // シード値設定
  // 調和振動子
  // E=1/2+3
  let d = |qt: T, _| 
    // (3(2Q(t)^2-1)/(2Q(t)^2-3)-Q(t)
    (3.*(2.*qt.powi(2)-1.)) /
    (    2.*qt.powi(2)-3.) 
    -qt
  ;
  // 見本経路40本
  (-20..=-1).chain(1..=20)
  .map(|q0| q0 as T * 0.5) // Q(t)初期値
  .for_each(|q0| {
    g.reset_data();
    seed = (seed+43) % 255; // ｼｰﾄﾞ値変更
    // 電子の自由運動
    qms2(d, q0, 0.05, 300, seed)
    // 見本経路描画
    .for_each(|(x, y)| {
       g.set_data(x, y); 
       graph_supply_draw!(g, display, Go1,);
    });
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}

