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
use itertools::{iproduct};
use linspacef32::linspacef32;
// キャビティ流れ
use cavityflow2d::*;
// グラフ供給
use graph_supply_quiver2d::*;
// ヒープアロケート関連
extern crate alloc;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
  // ヒープアロケート設定 -------------------
  {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024*70;
    static mut HEAP_MEM: 
      [MaybeUninit<u8>; HEAP_SIZE] = 
      [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(
      &raw mut HEAP_MEM as usize, HEAP_SIZE) 
    }
  }
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
       -10..210,     // x目盛
       -10..210,     // y目盛
       (100., 100.), // 補正率
       (50, 50),     // 目盛刻み
       true,         // ｶﾗｰﾏｯﾌﾟ適用
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 結果描画 -----------------------------
  let (nx, ny) = (41, 41); // グリッド数
  let (lx, ly) = (2., 2.); // x,y長さ
  cavityflow2d(
    nx, ny,           // グリッド数(x, y)
    50,               // 計算回数(圧力部分)
    1., 0.1,          // 密度, 粘性係数
    0.001,            // Δt
    lx/(nx-1) as f32, // Δx
    ly/(ny-1) as f32, // Δy
  )
  .step_by(10)
  .for_each(|(u, v)| {
    gb.draw(&mut display).unwrap();
    // meshgrid(ji)
    iproduct!(
      linspacef32(0., lx, nx).enumerate()
                             .step_by(2),
      linspacef32(0., ly, ny).enumerate()
                             .step_by(2)
    )
    .for_each(|((j, x), (i, y))| {
      // 可変長矢印
      let (a, norm) = make_argument2d(
        (x, y),                 // grid位置
        (u[(i, j)], v[(i, j)]), // ﾍﾞｸﾄﾙ
        1.2,                    // 長さ調整
      );
/*
      // 固定長矢印
      let (a, norm) = make_argument2d_fixed(
        (x, y),                 // grid位置
        (u[(i, j)], v[(i, j)]), // ﾍﾞｸﾄﾙ
        0.08,                   // 長さ調整
      );
*/
      g.set_data_arrow(a, norm);
      graph_supply_draw!(g, display, Go1,);
    });
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}


