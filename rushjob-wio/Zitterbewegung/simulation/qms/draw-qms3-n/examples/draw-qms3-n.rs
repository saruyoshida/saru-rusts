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
       0..200,     // x目盛
       -500..500,  // y目盛
       (10., 10.), // 補正率
       (50, 100),  // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0][0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  // 重みを変えた自由運動の重ね合わせ
  let mut seed: u8 = 0;       // シード値設定
  let px = 1.;                // 運動量
  // 加重比率10パターン毎に、
  // x初期値を0.2ずつ変えて見本経路を87本描画
  (0..10)                     // 10パターン
  .map(|aa| aa as T * 0.1) 
  .map(|aa| {
    let a  = aa.sqrt();       // 加重比率
    (2.*a*a-1., 2.*a*(1.-a*a).sqrt())
  })
  // 加重比率毎
  .for_each(|(a1, a2)| {
    // 描画領域クリア
    gb.draw(&mut display).unwrap();
    // 重ね合わせの平均前方微分DX(t)
    let d = move |xt: T, _| 
      2.*px*((a1-a2*(2.*px*xt).sin()) /
             (1.+a2*(2.*px*xt).cos()));
    // 見本経路87本描画
    (0..87)
    .map(|x0| x0 as T * 0.2)  // x初期値
    .for_each(|x0| {
      g.reset_data();
      seed = (seed+43) % 255; // ｼｰﾄﾞ値変更
      // 電子の自由運動
      qms(d, x0, 0.01, 2000, seed)
      // 見本経路描画
      .for_each(|(t, x)| {
         g.set_data(t, x); 
         graph_supply_draw!(g, display, Go1,);
      });
    });
    delay.delay_ms(2000u16);
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}

