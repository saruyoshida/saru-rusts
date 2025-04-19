#![no_std]
use rand_distr::{Distribution, Normal};
use rand::prelude::*;
use micromath::F32Ext;

type T = f32;
// 電子の自由運動
pub fn qms(
  d   : impl Fn(T, T) -> T+'static,  // DX(t)
  x0  : T,     // x初期値
  dt  : T,     // 時間増分
  cnt : usize, // 回数
  seed: u8,    // 乱数シード
) -> impl Iterator<Item=(T, T)> { // 結果位置
  // 正規分布乱数A(t)
  let mut rng = StdRng::from_seed([seed; 32]);
  let normal = Normal::new(0., 1.).unwrap();
  let mut at = move ||normal.sample(&mut rng);
  // 結果出力
  (0..cnt)
  .scan(x0, move |xt, c| {
     let x = *xt;
     let t = c as T * dt;
     // 確率微分方程式のx座標成分
     // x(t+Δt)=X(t)+DX(t)Δt+A(t)(√h/m=1)√Δt
     *xt += d(*xt, t)*dt+at()*dt.sqrt();
     Some((t, x))
  })
}