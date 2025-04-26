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
       0..300,      // x目盛
       -125..75,    // y目盛
       (100., 10.), // 補正率
       (100, 25),   // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0][0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // グラフ立体視化(補正率x,   y,   z)
  let p3 = Plot3D::new((100., 10., 40.));
  p3.draw(&mut display).unwrap();
  // 結果描画 -----------------------------
  let mut seed: u8 = 0; // シード値設定
  // スリット設定
  let a: T = 0.3;       // スリット巾
  let s: T = 0.3;       // ｽﾘｯﾄ間隔半値
  let aps  = a+s;       // a+s
  
  // 二重スリット(本)
  let d = move |yt:T, t: T| {
    let t2a2 = t*t+a*a;
    let ts   = t*s;
    // (2(t-a)/(t^2+a^2))Y(t) +
    // (ts/t^2+a^2)*tan(ts/t^2+a^2*Y(t))
    (2.*(t-a)) / t2a2 * yt +
    ts / t2a2 * (ts / t2a2 * yt).tan()
  };
  // 見本経路40本描画
  (0..40)
  .map(|z| z as T)
  .map(|z| (z, -aps+(1./20.)*aps*z))// y初期値
  .for_each(|(z, y0)| {
    g.reset_data();
    seed = (seed+43) % 255; // ｼｰﾄﾞ値変更
    // 電子の自由運動
    qms(d, y0, 0.01, 300, seed)
    // 見本経路描画
    .for_each(|(x, y)| {
       let (x, y) = p3.conv(
         x, y, (40.-z*1.5)/40.
       );
       g.set_data(x, y); 
       graph_supply_draw!(g, display, Go1,);
    });
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}

