
#![no_std]
#![no_main]

// 例
// サンプリングレート:83333.33
// ダウンサンプル数:4 
// ダウンサンプル後レート:20833.33
// ADCサンプル点数:2048
// FFTサンプル点数:512

// FFT後データ:spectrum[n]  n = 0〜Q/2 -1
// rfftはサンプル点数の半分の配列で、
// 複素数(実部と虚部)を返す。
// 返された複素数を(実部^2+虚部^2)のルートを
// 計算し、パワースペクトラムを取得する。
// .norm_sqr().sqrt()
// norm_sqrは実部^2+虚部^2を返す。
// ノルムの二乗という意味らしい。
//  
// 上記設定の場合、FFT後データは256点返って
// くる。(配列番号0 〜 255)
// 配列番号ごとに割当たる周波数は以下計算と
// なる。
// 配列番号 ✖️
// (ダウンサンプル後レート ➗
//  FFTサンプル点数)

// n = 15の場合
// 15 * (20833.33 / 512) = 610
// nひとつあたり、40.644
// 
// ハン窓関数
// h(n)=0.5-0.5 cos(2πn/(N-1))

// fft用配列:1024
// サンプリングデータ:4096 
// → バンドパスフィルタ 
//       カットオフ周波数:800(単位：Hz)
//       帯域幅:1(単位：octave)
// → データ間引き:1/4
//    ↓
//    ハン窓系数を掛けて、fft用配列に格納
//    ↓
//    rfft_1024
//    ↓
//    fft結果512個のデータに対し、
//    ノルムの二乗を取って最大値を取得する

//     build_bandpass_han_rfft

//  レ:1174.659
//  ミ:1318.510
//  ド:1046.502
//  ド:523.251
//  ソ:783.991

//  848を中心として上下1オクターブで
//  バンドパスフィルタ

use panic_halt as _;
use core::f32::consts::PI;
use micromath::F32Ext;
use wio_biquad::BiQuadFilter;

// ADCのサンプリングレート
const ADC_SAMPLING_RATE: f32 = 83333.0; 
// サンプリング点数
const SAMPLING_POINTS: u32 = 4096; 
// ダウンサンプル点数
const DOWN_FACTOR: u32 = 4; 
// FFTをするサンプル点数
const FFT_POINTS: u32 = 
  SAMPLING_POINTS / DOWN_FACTOR;
// ダウンサンプリング後のサンプリングレート
const SAMPLING_RATE: f32 = 
  ADC_SAMPLING_RATE / DOWN_FACTOR as f32;

pub struct WioFft {
  win_factor: [f32; FFT_POINTS as usize],
  biquad: BiQuadFilter,
}

impl WioFft {
  pub fn build_bandpass_down4_han_rfft(
    cutoff_fs: f32,
    cutoff_octave: f32,
  ) -> Self
  {
    let win_factor = 
      WioFft::hann_factor();
    let biquad = BiQuadFilter::build_bandpass(
      cutoff_fs,
      cutoff_octave,
      ADC_SAMPLING_RATE,
    );
    let wio_fft = WioFft {
      win_factor,
      biquad,
    };
    wio_fft
  }

  fn hann_factor() 
    -> [f32; FFT_POINTS as usize] 
  {
    let mut factor 
      = [0f32; FFT_POINTS as usize];
    for i in 0..FFT_POINTS as usize {
      factor[i] = 0.5f32 - 0.5f32
        * (PI * 2.0f32 * i as f32 / 
           FFT_POINTS as f32
          ).cos();
    }
    factor
  }

  fn pre_prosessing(
    &mut self,
    fft_buffer: &[f32],
    fft_result: &mut [f32], 
  ) 
  {
    let mut x = 0;
    let mut filterd_signal: f32;
    for i in 0..SAMPLING_POINTS as usize {
      filterd_signal = self.biquad.process(
        fft_buffer[i]
      );
      if i as u32 % DOWN_FACTOR == 0 {
        fft_result[x] = filterd_signal
          * self.win_factor[x];
        x += 1;
      }
    }
  }

  pub fn get_pitch(
    &mut self,
    fft_buffer: &[f32],
    fft_threshold: f32
  ) -> Option<(f32, f32)>
  {
    let mut max = 0f32;
    let mut num = 0u32;

    let mut fft_result 
       = [0f32; FFT_POINTS as usize];

    self.pre_prosessing(
      fft_buffer,
      &mut fft_result,
    );

    let result = microfft::real::rfft_1024(
      &mut fft_result
    );

    for (step, spectrum) in 
      result.iter().enumerate() 
    {
      let power = 
        spectrum.norm_sqr().sqrt();


      if power > max 
       && power > fft_threshold
      {
        max = power;
        num = step as u32;
      }
    }

    let mut freqnum: f32 = num as f32;

    if num > 0 &&
       num < FFT_POINTS / 2 - 1
    {
       let dleft: f32 = 
         result[(num - 1) as usize]
         .norm_sqr().sqrt() / max;

       let dright: f32 = 
         result[(num + 1) as usize]
         .norm_sqr().sqrt() / max;

       freqnum += 0.5f32 *
       (dright * dright - dleft * dleft);
    }

    freqnum *= 
      SAMPLING_RATE / FFT_POINTS as f32;

    if num == 0 ||
       num == FFT_POINTS / 2 - 1
    {
      None
    } else {
      Some((freqnum, max))
    }
  }
}

