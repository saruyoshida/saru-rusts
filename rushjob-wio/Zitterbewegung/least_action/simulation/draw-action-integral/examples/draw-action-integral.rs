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
  primitives::{
    Rectangle,
    PrimitiveStyle,
  },
  pixelcolor::Rgb565,
  prelude::*,
};

use micromath::F32Ext;

// 作用積分
use action_integral::*;
// グラフ供給
use graph_supply_action_integral::*;

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
  // グラフ供給 ---------------------------
  let (mut graph_box, mut graph_obj) =
                    graph_supply();
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
  graph_box.iter_mut().for_each(|g| 
    g.mode_scale().draw(&mut display).unwrap()
  );
  // 結果描画 -----------------------------
  graph_obj.iter_mut().for_each(|g| 
    trial_action().for_each(|(st, k)| {
      g[0].set_data(k, st); 
      g.iter().for_each(|go|  
        // graph_supply_drawマクロによる実装
        graph_supply_draw!(go, display, Go1,)
      )
    })
  );
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}

type T = f32;

// 振動子のﾗｸﾞﾗﾝｼﾞｱﾝ
pub fn oscillator(
  m: T,    // 質量
  k: T,    // バネ定数
) -> impl Fn(T, T) -> T {
  // 1/2Mv^2 - 1/2Kx^2
  move |x, v| 0.5 * (m*v*v - k*x*x)
}

// 試行関数の変分
fn trial_action() 
  -> impl Iterator<Item=(T, T)>  {
  let m  = 1.0;      // 質量
  let w  = 2.5;      // √バネ定数
  let t0 = 0.0;      // 積分範囲(始端)
  let t1 = 1.0;      // 積分範囲(終端)
  let ndiv = 200;    // 分割数
  let dt   = 0.0001; // Δt

  // 変分範囲の写像
  (15..28)
  .map(|k| k as T * 0.1)
  .map(move |k| (
     // 対象関数x(t):sin(kt)/sin(kt1)
     move |t: T| (k*t).sin() / (k*t1).sin(),
     k,
  ))
  .map(move |(xt, k)| (
     // 作用:L(x(t), x'(t))を関数合成
     action(
       oscillator(m, w*w),   // ﾗｸﾞﾗﾝｼﾞｱﾝ
       xt,                   // 対象関数
       differential(dt, xt), // 対象関数微分
     ),
     k,
  ))
  .map(move |(action, k)| (
     // 積分:∫t0→t1 L(x(t), x'(t))
     integral(
       t0,                 // 積分範囲(始端)
       t1,                 // 積分範囲(終端)
       ndiv,               // 分割数
       action,             // 作用
     ), 
     k,
  ))
}


