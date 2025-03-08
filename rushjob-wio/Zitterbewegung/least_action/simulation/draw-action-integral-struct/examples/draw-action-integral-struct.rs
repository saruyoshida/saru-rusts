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

// 作用積分
use action_integral_struct::*;
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

// 試行関数の変分
fn trial_action() 
  -> impl Iterator<Item=(T, T)>  {
  let mut action = create_trial_action(2.5);

  (15..28)
  .map(|k| k as T * 0.1)
  .map(move |k| (
     action.set_k(k).action_integral(),
     k,
  ))
}

