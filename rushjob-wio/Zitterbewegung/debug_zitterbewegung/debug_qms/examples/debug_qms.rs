use chrono::{Local, Timelike};
use qms::*;

type T = f32;
// 電子の自由運動
fn main() {
  // 乱数シード値設定
  let time = Local::now();
  let seed = (time.nanosecond() % 256) as u8;
  // 平均前方微分DX(t)
  // ※DX(t)=Px/m→電子の速度
  //  →Px:電子の運動量,m:電子の質量
  //  →原子単位系を用いPx=m=1
  let d1 = move |_, _|  1.;
  // ※反対向き
  let d2 = move |_, _| -1.;
  // ※重ね合わせ
  let d3 = move |xt: T, _| -xt.tan();
  // ※重みを変えた自由運動の重ね合わせ
  let px = 1.;                // 運動量
  let a  = (0.1 as T).sqrt(); // 加重比率
  let a1 = 2.*a*a-1.;
  let a2 = 2.*a*(1.-a*a).sqrt();
  let d4 = move |xt: T, _| 
    2.*px*((a1-a2*(2.*px*xt).sin()) /
           (1.+a2*(2.*px*xt).cos()));
  // 結果出力
  println!("結果リスト");
  result_list(d1, seed, 5000, 1000);
  println!("反対向き");
  result_list(d2, seed, 5000, 1000);
  println!("重ね合わせ");
  result_list(d3, seed, 5000, 1000);
  println!("重みを変えた重ね合わせ");
  result_list(d4, seed, 2000, 400);
}
// 結果リスト
fn result_list(
  d   : impl Fn(T, T) -> T+'static, // D(xt)
  seed: u8,    // 乱数シード
  cnt : usize, // 回数
  step: usize, // 観測ステップ
) {
  qms(d, 0., 0.01, cnt, seed)
  .step_by(step).for_each(|(t, x)| 
     println!("t:{},x:{}", t, x)
  );
}