
#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use core::fmt::Write;
use cortex_m::peripheral::NVIC;
use heapless::consts::*;
use heapless::Vec;
use heapless::String;
use wio::entry;
use wio::hal::adc::{FreeRunning, 
                    InterruptAdc};
use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{interrupt, CorePeripherals, 
               Peripherals, ADC1};
use wio::prelude::*;
use wio::Pins;

use wio_fft::WioFft;
use wio_term::Terminal;

struct Ctx {
  adc: InterruptAdc<ADC1, FreeRunning>,
  buffers: [SamplingBuffer; 2], 
  // ADC結果のバッファ2面分
  // 現在ADC結果取り込み先のバッファへの参照
  sampling_buffer: 
    Option<&'static mut SamplingBuffer>,
  // 現在信号処理中のバッファへの参照
  processing_buffer: 
    Option<&'static mut SamplingBuffer>,
}

static mut CTX: Option<Ctx> = None;

type SamplingBuffer = 
  heapless::Vec<f32, U4096>; 
//サンプリングバッファの型

#[entry]
fn main() -> ! {
  let mut peripherals = 
    Peripherals::take().unwrap();
  let core = CorePeripherals::take().unwrap();
  let mut clocks = 
    GenericClockController::
      with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

  let mut sets = Pins::new(peripherals.PORT)
                 .split();
  let mut delay = Delay::new(
                  core.SYST, &mut clocks);

  // 画面を初期化する
  let (display, _backlight) = sets
    .display
    .init(
       &mut clocks,
       peripherals.SERCOM7,
       &mut peripherals.MCLK,
       &mut sets.port,
       60.mhz(),
       &mut delay,
     )
     .unwrap();

  let mut terminal = Terminal::new(display);

  //ようにInterruptAdc型を構築する
  let (microphone_adc, mut microphone_pin) = 
    sets.microphone.init(
      peripherals.ADC1,
      &mut clocks,
      &mut peripherals.MCLK,
      &mut sets.port,
    );
  let mut microphone_adc: 
    InterruptAdc<_, FreeRunning> =
      InterruptAdc::from(microphone_adc);

  // ADCの変換処理を開始する
  microphone_adc.start_conversion(
    &mut microphone_pin);

//  terminal.write_str("Terminal build\n");

//  let mut wio_fft = WioFft:: 
//    build_lowpass_down4_han_rfft(
//      1700f32, 1f32 / 1.41421f32
//    );

  let mut wio_fft = WioFft:: 
    build_bandpass_down4_han_rfft(
      880f32, 1f32
    );

//  terminal.write_str("wio_fft build\n");

  let mut textbuffer = String::<U256>::new();

  // 共有リソースを初期化する
  unsafe {
    CTX = Some(Ctx {
      adc: microphone_adc,
      buffers: [Vec::new(), Vec::new()],
      sampling_buffer: None,
      processing_buffer: None,
    });
    // 2面分のサンプリングバッファを
    //取り込み用と処理用にそれぞれ割り当てる
    let mut ctx = CTX.as_mut().unwrap();
    let (first, rest) = ctx
      .buffers.split_first_mut().unwrap();
    ctx.sampling_buffer = Some(first);
    ctx.processing_buffer = 
      Some(&mut rest[0]);
  }

// ADC変換完了割り込み(RESRDY)を
// 有効にしてサンプリングを開始する
  unsafe { 
    NVIC::unmask(interrupt::ADC1_RESRDY); 
  }
 
//  terminal.write_str("loop star\n");

  loop 
  {
    // safe: processing_bufferは、
    // mainループでclearするまで
    // 割り込みハンドラが触らないので安全
    let processing_buffer = unsafe {
      let ctx = CTX.as_mut().unwrap();
      ctx.processing_buffer
        .as_mut().unwrap()
    };

    let len = processing_buffer.len();
    let cap = processing_buffer.capacity();

    // 処理対象バッファにFFT点数分の
    // サンプルデータが入っている？
    if len != cap { continue; }
    
    if let Some((freq, maxv)) =
      wio_fft.get_pitch(
       &processing_buffer,
       25000.0f32
      )
    {
      textbuffer.clear();
      writeln!(
        textbuffer, "{}Hz, {}", freq, maxv
      ).unwrap();
      terminal.write_str(
        textbuffer.as_str()
      );
    }

    processing_buffer.clear();
  }
}

#[interrupt]
fn ADC1_RESRDY() 
{
  unsafe {
    let ctx = CTX.as_mut().unwrap();
    if let Some(sample) = 
      ctx.adc.service_interrupt_ready()
    {
      let sampling_buffer = 
        ctx.sampling_buffer
          .as_mut().unwrap();
      if sampling_buffer.len() == 
         sampling_buffer.capacity() {
        if ctx.processing_buffer
           .as_mut().unwrap().len() == 0      
        {
          core::mem::swap(
            &mut ctx.processing_buffer,
            &mut ctx.sampling_buffer,
          );
        }
      } else { 
        let _ = sampling_buffer.push(
          sample as f32
        );
      }
    }
  }
}