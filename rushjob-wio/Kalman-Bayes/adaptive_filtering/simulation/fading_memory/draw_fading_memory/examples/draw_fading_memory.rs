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
// フィルタ供給
use filter_supply_fading_memory::*;
// グラフ供給
use graph_supply_fading_memory::*;
// シミュレーションデータ
use generate_data::GenerateData;

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
  // フィルタ/グラフ供給 -----------------
  let mut filters = filter_supply();
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
  // 繰り返し -----------------------------
  let dt = 0.1;
  let gd = GenerateData::new(70, 1.2);
  for (i, (_, z)) in gd.into_iter()
                         .enumerate() {
    for (f, g) in filters.iter_mut()
                  .zip(graph_obj.iter_mut())
    { 
      // フィルタ操作
      f.z_set(0, z.0);  // 観測値設定
      f.predict();      // 予測
      f.update();       // 更新
      // グラフ描画
      g[0].set_data(i as f32 * dt, z.0); 
      g[1].set_data(i as f32 * dt, f.x(0)); 
      g.iter().for_each(|go| 
        // graph_supply_drawマクロによる実装
        graph_supply_draw!(
          go, display, Go1, Go2,
        )
      );
    }
  }
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}


