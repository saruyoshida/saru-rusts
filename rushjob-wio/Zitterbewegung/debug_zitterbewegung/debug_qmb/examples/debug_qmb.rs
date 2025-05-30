use qms::*;
use core::f32::consts::PI;

type T = f32;
fn main() {
  // 乱数シード値設定
  let seed = 43u8;
  // バリアー散乱
  let d1 = |xt:T, _| {
    if xt < 0. {
      // X(t) <  wall の場合
      // -tan(2X(t)-(3/2)π)
      -(2.*xt-1.5*PI).tan()
    } else {
      // X(t) >= wall の場合
      // -1/2
      -0.5
    }
  };
  // 結果出力
  println!("バリアー散乱");
  result_list(d1, seed, 20000, 4000);
}
// 結果リスト
fn result_list(
  d   : impl Fn(T, T) -> T+'static, // D(xt)
  seed: u8,    // 乱数シード
  cnt : usize, // 回数
  step: usize, // 観測ステップ
) {
  qms2(d, -10., 0.01, cnt, seed)
  .step_by(step).for_each(|(t, x)| 
     println!("t:{},x:{}", t, x)
  );
}