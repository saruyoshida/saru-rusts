#![no_std]
#![no_main]

// テキスト表示テスト
use panic_halt as _;
use wio_terminal as wio;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Pins, Sets};

use emb_textterm::*;

// 表示設定
const BASE_COLOR : Rgb565 = Rgb565::BLACK;
const BOX_COLOR  : Rgb565 = Rgb565::BLACK;
const TEXT_COLOR : Rgb565 = Rgb565::WHITE;
const TXT2_COLOR : Rgb565 = Rgb565::CYAN;


#[entry]
fn main() -> ! {
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

  let mut et = EmbTextterm::new(
    (10, 10),            // 表示開始位置
    (158_u32, 220_u32),  // 表示サイズ
  );

  et.set_base_color(BASE_COLOR)
    .set_text_color(TEXT_COLOR)
    .set_txt2_color(TXT2_COLOR)
    .set_box_color(BOX_COLOR);

  let mut datas =  [
    "a\n",
    "ABCDEFG\n",    "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ\n",
    "\n",
    "",
    "@&¥)",
    "987654321",
    "zyx\n",
  ];

  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  )
  .into_styled(
    PrimitiveStyle::with_fill(BASE_COLOR)
  )
  .draw(&mut display)
  .unwrap();

  et.mode_allclear()
    .draw(&mut display)
    .unwrap();

  et.mode_clear().draw(&mut display).unwrap();
  let mut cnt = 0;

  loop {
    et.mode_data();
    for d in datas.into_iter()
    {
      et.set_data(EttString::from(d))
        .draw(&mut display)
        .unwrap();

      delay.delay_ms(100 as u16);
    }

    datas.rotate_left(1);
    cnt += 1;
    if cnt % 13 == 0 {
      et.mode_reset()
        .draw(&mut display)
        .unwrap();
      cnt = 0;
    }
  }
}

