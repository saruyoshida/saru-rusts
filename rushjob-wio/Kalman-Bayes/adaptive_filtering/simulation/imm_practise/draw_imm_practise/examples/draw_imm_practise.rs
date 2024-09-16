#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;
use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use embedded_graphics::{
  primitives::{
    Rectangle,
    PrimitiveStyle,
  },
  pixelcolor::Rgb565,
  prelude::*,
};
// IMM供給
use imm_supply_imm_practise::*;
// グラフ供給
use graph_supply_imm_practise::*;
// シミュレーションデータ
use turning_target::TurningTarget;

#[entry]
fn main() -> ! {
  // wio_terminal設定 -----------------------
  let mut peripherals = 
    Peripherals::take().unwrap();
  let core = 
    CorePeripherals::take().unwrap();

  let mut clocks = GenericClockController::
    with_external_32kosc(
      peripherals.GCLK,
      &mut peripherals.MCLK,
      &mut peripherals.OSC32KCTRL,
      &mut peripherals.OSCCTRL,
      &mut peripherals.NVMCTRL,
  );

  let mut delay = Delay::new(
    core.SYST, &mut clocks);
  let pins = Pins::new(peripherals.PORT);
  let mut sets: Sets = pins.split();

  let (mut display, _backlight) = 
    sets.display
      .init(
        &mut clocks,
        peripherals.SERCOM7,
        &mut peripherals.MCLK,
        &mut sets.port,
        58.mhz(),
        &mut delay,
      )
      .unwrap();
  // IMM/グラフ供給 -----------------
  let mut imm = IMMSupply::new();
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
  graph_box.iter_mut().for_each(|g| 
    g.mode_scale().draw(&mut display).unwrap()
  );
  // シミュレーション設定------------------
  let n         : usize = 600;
  let turn_start: usize = 400;
  let dt        : f32   = 1.0;
  let std       : f32   = 1.0;
  let tg = TurningTarget
          ::new(n, turn_start, dt, std);
  // 繰り返し -----------------------------
  for (i, z) in tg.into_iter().enumerate() {
    // IMM操作
    imm.z.copy_from_slice(&z); // 観測値設定
    imm.predict();             // 予測
    imm.update();              // 更新
    // グラフ描画
    let t = i as f32 * dt;
    g[0].set_data(z[0], z[1]); 
    g[1].set_data(
           imm.imm.x[(0, 0)],
           imm.imm.x[(3, 0)]
    );
    g[2].set_data(t, imm.imm.mu[(0, 0)]); 
    g[3].set_data(t, imm.imm.mu[(1, 0)]); 
    g.iter().for_each(|go| 
      // graph_supply_drawマクロによる実装
      graph_supply_draw!(
        go, display, Go1, Go2,
      )
    );
  }
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}


