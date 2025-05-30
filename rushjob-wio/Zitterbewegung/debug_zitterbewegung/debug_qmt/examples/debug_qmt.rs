use qms::*;
use core::f32::consts::PI;
use std::env::args;

type T = f32;
fn main() {
  // シード初期値引数設定
  let args: Vec<String> = args().collect();
  let mut seed = args[1]
                 .parse::<u8>().unwrap();
  // 量子トンネル効果
  let wall = 0.5;         // 隔壁厚み/2
  let d = move |xt: T, _| {
    if xt < -wall {
      // X(t) < -wall の場合
      // 2*
      // (4.245386-2sin(2X(t)+1-1/2π)) /
      // (4.692899+2cos(2X(t)+1-1/2π))
      2.*
      (4.245386-2.*(2.*xt+1.-0.5*PI).sin()) /
      (4.692899+2.*(2.*xt+1.-0.5*PI).cos())
    } else if xt > wall {
      // X(t) > wall の場合
      1.
    } else {
      // -wall <= X(t) <= wall の場合
      // 2(tanh(2X(t)-1)+1/cosh(2X(t)-1))
      2.*(   (2.*xt-1.).tanh() +
          1./(2.*xt-1.).cosh())
    }
  };
  (0..10)
  .for_each(|_| {
     // 結果出力
     println!("量子トンネル効果({})", seed);
     result_list(d, seed, 31, 6);
     seed += 1u8;
  });
}
// 結果リスト
fn result_list(
  d   : impl Fn(T, T) -> T+'static, // DX(t)
  seed: u8,    // 乱数シード
  cnt : usize, // 回数
  step: usize, // 観測ステップ
) {
  qms2(d, -1., 0.01, cnt, seed)
  .step_by(step).for_each(|(t, x)| 
     println!("t:{},x:{}", t, x)
  );
}