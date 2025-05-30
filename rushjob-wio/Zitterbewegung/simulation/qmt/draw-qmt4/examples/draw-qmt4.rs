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
use core::f32::consts::PI;

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
       0..2000,     // x目盛
       -600..1200,  // y目盛
       (10., 10.),  // 補正率
       (500, 200),  // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0][0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  // 乱数シード値設定
  let mut seed = 43u8;
  // 量子トンネル効果
  let wall = 2.;         // 隔壁厚み/2
  let d = move |xt: T, _| {
    if xt < -wall {
      // X(t) < -wall の場合
      // 2(sin(4.245386-2(2X(t)+1-0.5π)) /
      //   cos(4.692899+2(2X(t)+1-0.5π)))
      2.*
      (4.245386-2.*(2.*xt+1.-0.5*PI).sin()) /
      (4.692899+2.*(2.*xt+1.-0.5*PI).cos())
    } else if xt > wall {
      // X(t) > wall の場合
      1.
    } else {
      // -wall <= X(t) <= wall の場合
      // 2(tanh(2X(t)-1)+(1/cosh(2X(t)-1))
      2.*(   (2.*xt-1.).tanh() +
          1./(2.*xt-1.).cosh())
    }
  };
  // 10本表示
  (0..10)
  .for_each(|_| {
    g.reset_data();
    seed = (seed+43) % 255; // ｼｰﾄﾞ値変更
    // 電子の自由運動
    qms2(d, -40., 0.01, 20000, seed)
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

