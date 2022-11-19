#![no_std]
#![no_main]

//  レ:1174.659    roght
//  ミ:1318.510    left
//  ド:1046.502    down
//  ド:523.251     toplft
//  ソ:783.991     topmidol
//  レ:587.330     up
//  ミ:659.255     click

use panic_halt as _;
use wio_terminal as wio;

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::
  {PrimitiveStyle, Rectangle};

use cortex_m::interrupt::{
    free as disable_interrupts, 
};
use cortex_m::peripheral::NVIC;

use wio::hal::clock::GenericClockController;
use wio::hal::delay::Delay;
use wio::pac::{interrupt, CorePeripherals, 
               Peripherals, ADC1};
use wio::hal::adc::{FreeRunning, 
                    InterruptAdc};
use wio::prelude::*;
use wio::{entry, Pins, Sets};
use wio::hal::rtc;

use heapless::consts::*;
use heapless::String;
use heapless::Vec;

use wio_clock::WioClock;
use wio_buttons::{WioButtons};
use wio_toast::WioToast;
use wio_fft::WioFft;
use wio_fftbutton::WioFftButton;

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

//サンプリングバッファの型
type SamplingBuffer = 
  heapless::Vec<f32, U4096>; 


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

  let rtc = rtc::Rtc::clock_mode(
    peripherals.RTC, 
    1024.hz(), 
    &mut peripherals.MCLK
  );

  unsafe {
    RTC = Some(rtc);
  }

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

  //InterruptAdc型を構築する
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

  let mut wio_clock = WioClock::new(
    &mut display
  );

  let mut wio_button = WioButtons::build(
    (0i32, 0i32, 23i32, 1i32, 
      String::from("Hou")),
    (0i32, 0i32, 59i32, 1i32, 
      String::from("Min")),
    (0i32, 0i32, 59i32, 1i32, 
      String::from("Sec"))
  );


  let mut wio_toast = WioToast::new(
    1200000u32,
    Point::new(260, 210),
    Size::new(60, 20),
    Rgb565::BLACK,
    Rgb565::WHITE,
    Rgb565::WHITE,
  );

  let mut wio_fft = WioFft:: 
    build_bandpass_down4_han_rfft(
      880f32, 2f32
  );

  let mut wio_fftbutton = WioFftButton::new(
    523.251f32,  //  ド:topleft
    783.991f32,  //  ソ:topmiddle
    1046.502f32, //  ド:down
    587.330f32,  //  レ:up
    1318.510f32, //  ミ:left
    1174.659f32, //  レ:right
    659.255f32,  //  ミ:click
    10f32,       //  許容誤差
  );

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

  Rectangle::new(
    Point::new(0, 0), 
    Size::new(320, 240),
  )
  .into_styled(
    PrimitiveStyle::with_fill(
      Rgb565::WHITE)
  )
  .draw(&mut display).unwrap();


  loop {
//        delay.delay_ms(1000 as u16);
    let time =
      disable_interrupts(|_| unsafe { 
        RTC.as_mut().map(|rtc| 
          rtc.current_time()
        ) 
      }
    ).unwrap();

    wio_clock.update(
     time.hours,
     time.minutes, 
     time.seconds
    );
    
    wio_clock.draw(&mut display).unwrap();

    wio_toast.count_down();
    wio_toast.draw(&mut display).unwrap();

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
    
    if let Some((freq, _maxv)) =
      wio_fft.get_pitch(
       &processing_buffer,
       25000.0f32
      )
    {
     
      if let Some(press) = 
        wio_fftbutton.get_button(freq)
      {
        wio_button.reset_value(
          (time.hours as i32,
           time.minutes as i32,
           time.seconds as i32)
        );

        wio_toast.start(
          String::from(
            wio_button.get_state().as_str()
          )
        );
      
        if let Some(now_time) =
          wio_button.button_pulled(press)
        {
          set_time(now_time.0 as u8,
                   now_time.1 as u8,
                   now_time.2 as u8);
        }
      
      }

    }

    processing_buffer.clear();
  }
}

static mut RTC: Option<rtc::Rtc<rtc::ClockMode>> = None;

fn set_time(hour: u8, minute: u8, second: u8) {
  disable_interrupts(|_| {
    unsafe {
      RTC.as_mut().map(|rtc| {
        rtc.set_time(rtc::Datetime {
          seconds: second,
          minutes: minute,
          hours: hour,
          day: 0,
          month: 0,
          year: 0,
        });
      });
    }
  });
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

