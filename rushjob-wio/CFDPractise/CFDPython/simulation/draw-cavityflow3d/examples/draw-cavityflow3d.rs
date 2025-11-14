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

// ヒープアロケート関連
extern crate alloc;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

// use num_traits::Float;
use itertools::iproduct;
use linspacef32::linspacef32;

// キャビティ流れ
use cavityflow3d::*;
// グラフ供給
use graph_supply_quiver3d::*;

type T = f32;
type U = usize;
#[entry]
fn main() -> ! {
  // ヒープアロケート設定 -------------------
  {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024*80;
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
  let (mut graph_box, 
       mut graph_obj,
       mut graph_obj2) =
    graph_supply(
       0..200,       // x目盛
       0..200,       // y目盛
       (100., 100.), // 補正率
       (200, 200),   // 目盛刻み
       true,         // ｶﾗｰﾏｯﾌﾟ適用
    );
  let gb = &mut graph_box[0];
  let ga = &mut graph_obj;
  let gl = &mut graph_obj2;
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // 3d化     -----------------------------
  let (mut p3, mut eye) = Plot3D::new(
    2., 2., 2., (100., 100., 100.)
  );
  // 初期視点設定
  eye.right().down().down().out();
  p3.sb.set_eye(eye.position()); // 視点設定
  // 結果描画 -----------------------------
  let (nx, ny, nz) = (11, 11, 11);
  cavityflow3d(
    nx, ny, nz,      // グリッド数(x, y, z)
    20,              // 収束判定回数(圧力部分)
    40.,             // ﾚｲﾉﾙｽﾞ数
    0.01,            // Δt
    1./(nx-1) as T,  // Δx
    1./(ny-1) as T,  // Δy
    1./(nz-1) as T,  // Δz
    1e-5,            // 誤差閾値
  )
  .step_by(10)
  .for_each(|(u, v, w)| {

    // 視点回転 --
    eye.right();                   // 視点変更
    p3.sb.set_eye(eye.position()); // 視点設定

    // 画面ｸﾘｱ
    gb.draw(&mut display).unwrap();
    // グラフ枠表示
    gl.reset_data();
    ruled_line()
    .for_each(|(line, color)| {
      let (x, y) = p3.conv(line);
      gl.set_data(x, y);
      gl.set_color(color);
      graph_supply_draw!(
        gl, display, Go1, Go2,
      );
    });
    // メッシュグリッド
    meshgrid(nx, ny, nz, eye.hw, 1)
    .for_each(|((k, y), (j, z), (i, x))| {
      // 可変長矢印　始点終点取得
      let (p, norm) = make_argument3d(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        1.,                    // 長さ調整
      );
/*
      // 固定長長矢印　始点終点取得
      let (p, norm) = make_argument3d_fixed(
        (x, y, z),              // grid位置
                                // ﾍﾞｸﾄﾙ
        (u[(i,j,k)], v[(i,j,k)], w[(i,j,k)]),
        0.12,                   // 長さ調整
      );
*/
      // 始点、終点立体視化
      let (start, end) = (
        p3.conv(p.0), p3.conv(p.1)
      );
      // 矢印グラフ描画
      ga.set_data_arrow((start, end), norm);
      graph_supply_draw!(
        ga, display, Go1, Go2,
      );
    });
    //delay.delay_ms(1500_u16);
  });
  // -------------------------------------
  #[allow(clippy::empty_loop)]
  loop {}
}
// 手抜きな描画順設定
use auto_enums::auto_enum;
#[auto_enum(Iterator)]
fn meshgrid(
  nx: U, ny: U, nz: U, // x, y, z
  hw: i32, step:U      // カメラ角度, ステップ
)-> impl Iterator<Item=((U, T),(U, T),(U, T))>
{
  let (sx, sy, sz) = (0., 0., 0.);
  let (ex, ey, ez) = (2., 2., 2.);

  if (0..90).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate().rev()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate()
                           .step_by(step),
    )
  } else if (90..180).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate()
                           .step_by(step),
    )
  } else if (180..270).contains(&hw) {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate().rev()
                           .step_by(step),
    )
  } else {
    iproduct!(
      linspacef32(sy,ey,ny).enumerate()
                           .step_by(step),
      linspacef32(sz,ez,nz).enumerate().rev()
                           .step_by(step),
      linspacef32(sx,ex,nx).enumerate().rev()
                           .step_by(step),
    )
  }
}
