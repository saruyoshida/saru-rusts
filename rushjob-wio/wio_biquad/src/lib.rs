#![no_std]

use panic_halt as _;
use micromath::F32Ext;
use core::f32::consts::PI;

pub struct BiQuadFilter
{
  a0: f32,
  a1: f32,
  a2: f32,
  b0: f32,
  b1: f32,
  b2: f32,
  out1: f32,
  out2: f32,
  in1: f32,
  in2: f32,
}

impl BiQuadFilter
{
  // 入力信号にフィルタを適用する関数
  pub fn process(&mut self, in_signal: f32) 
    -> f32
  {
    // 入力信号にフィルタを適用し、
    //出力信号変数に保存。
    let out: f32 = 
      self.b0 / self.a0 * in_signal +
      self.b1 / self.a0 * self.in1  +
      self.b2 / self.a0 * self.in2  - 
      self.a1 / self.a0 * self.out1 - 
      self.a2 / self.a0 * self.out2;

    // 2つ前の入力信号を更新
    self.in2 = self.in1; 
    // 1つ前の入力信号を更新
    self.in1 = in_signal;  
    // 2つ前の出力信号を更新
    self.out2 = self.out1; 
    // 1つ前の出力信号を更新
    self.out1 = out;  
 
    // 出力信号を返す
    out
  }

  // ローパスフィルタビルダ
  pub fn build_lowpass(
    freq: f32,
    q: f32, 
    samplerate: f32,
  ) -> Self 
  {
    // フィルタ係数計算で使用する中間値を
    // 求める。
    let omega: f32 = 
      2.0f32 * PI * freq / samplerate;
    let alpha: f32 = 
    omega.sin() / (2.0f32 * q);
 
    let biquad_filter = BiQuadFilter {
      // フィルタ係数を求める。
      a0: 1.0f32  + alpha,
      a1: -2.0f32 * omega.cos(),
      a2: 1.0f32  - alpha,
      b0: (1.0f32 - omega.cos()) /
              2.0f32,
      b1: 1.0f32 -  omega.cos(),
      b2: (1.0f32 - omega.cos()) /
              2.0f32,
      in1: 0.0f32,
      in2: 0.0f32,
      out1: 0.0f32,
      out2: 0.0f32,
    };

    biquad_filter
  }

  // ハイパスフィルタビルダ
  pub fn build_highpass(
    freq: f32,
    q: f32, 
    samplerate: f32,
  ) -> Self 
  {
    // フィルタ係数計算で使用する中間値を
    // 求める。
    let omega: f32 = 
      2.0f32 * PI * freq / samplerate;
    let alpha: f32 = 
      omega.sin() / (2.0f32 * q);
 
    let biquad_filter = BiQuadFilter {
    // フィルタ係数を求める。
      a0: 1.0f32  + alpha,
      a1: -2.0f32 * omega.cos(),
      a2: 1.0f32  - alpha,
      b0: (1.0f32 + omega.cos()) /
              2.0f32,
      b1: -(1.0f32 + omega.cos()),
      b2: (1.0f32 + omega.cos()) /
              2.0f32,
      in1: 0.0f32,
      in2: 0.0f32,
      out1: 0.0f32,
      out2: 0.0f32,
    };

    biquad_filter
  }
 
  // バンドパスフィルタビルダ
  pub fn build_bandpass(
    freq: f32,
    bw: f32, 
    samplerate: f32,
  ) -> Self 
  {
    // フィルタ係数計算で使用する中間値を
    // 求める。 
    let omega: f32 = 
      2.0f32 * PI * freq / samplerate;
    let alpha: f32 = omega.sin() *
      BiQuadFilter::sinh(
        2.0f32.ln() / 2.0f32 * bw * 
        omega / omega.sin()
      );
 
    let biquad_filter = BiQuadFilter {
      // フィルタ係数を求める。
      a0: 1.0f32 + alpha,
      a1: -2.0f32 * omega.cos(),
      a2: 1.0f32 - alpha,
      b0: alpha,
      b1: 0.0f32,
      b2: -alpha,
      in1: 0.0f32,
      in2: 0.0f32,
      out1: 0.0f32,
      out2: 0.0f32,
    };

    biquad_filter
  }

  pub fn reset(&mut self)
  {
    self.in1 = 0.0f32;
    self.in2 = 0.0f32;
    self.out1 = 0.0f32;
    self.out2 = 0.0f32;
  }

// sinh(x) = (e^x - e^(-x)) / 2
// cosh(x) = (e^x + e^(-x)) / 2
  fn sinh(x: f32) -> f32
  {
    (x.exp() - (-x.exp())) / 2.0f32
  }
}
 



