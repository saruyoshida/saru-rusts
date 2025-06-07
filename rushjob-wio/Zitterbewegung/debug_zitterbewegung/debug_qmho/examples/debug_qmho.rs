use qms::*;
use chrono::{Local, Timelike};

type T = f32;
fn main() {
  // 乱数シード値設定
  let time = Local::now();
  let seed = (time.nanosecond() % 256) as u8;
  
  // 調和振動子
  // E=1/2 基底状態
  let d0 = |qt: T, _|        -qt;
  // E=1/2+1
  let d1 = |qt: T, _| 1.0/qt -qt;
  // E=1/2+2 
  let d2 = |qt: T, _| 
    // 4Q(t)/(2Q(t)^2-1)-Q(t)
    4.*qt/(2.*qt.powi(2)-1.)
    -qt
  ;
  // E=1/2+3
  let d3 = |qt: T, _| 
    // (3(2Q(t)^2-1)/(2Q(t)^2-3)-Q(t)
    (3.*(2.*qt.powi(2)-1.)) /
    (    2.*qt.powi(2)-3.) 
    -qt
  ;
  // E=1/2+4
  let d4 = |qt: T, _| 
    // (8(2Q(t)^2-3)Q(t))  /
    // (  4Q(t)^4-12Q(t)^2+3)
    // - Q(t)
    (8.*(2.*qt.powi(2)-3.)*qt) /
    (    4.*qt.powi(4)-12.*qt.powi(2)+3.)
    -qt
  ;
  // E=1/2+5
  let d5 = |qt: T, _| 
    // (20Q(t)^4-60Q(t)^2+15) /
    // ((4Q(t)^4-20Q(t)^2+15)*Q(t))
    //  - Q(t)
    (20.*qt.powi(4)-60.*qt.powi(2)+15.) /
    ((4.*qt.powi(4)-20.*qt.powi(2)+15.)*qt)
    -qt
  ;
  // 結果出力
  println!("調和振動子");
  println!("E=1/2 基底状態");
  result_list(d0, seed, 301, 60);
  println!("E=1/2+1");
  result_list(d1, seed, 301, 60);
  println!("E=1/2+2");
  result_list(d2, seed, 301, 60);
  println!("E=1/2+3");
  result_list(d3, seed, 301, 60);
  println!("E=1/2+4");
  result_list(d4, seed, 301, 60);
  println!("E=1/2+5");
  result_list(d5, seed, 301, 60);
}
// 結果リスト
fn result_list(
  d   : impl Fn(T, T) -> T+'static, // DX(t)
  seed: u8,    // 乱数シード
  cnt : usize, // 回数
  step: usize, // 観測ステップ
) {
  qms2(d, -10., 0.05, cnt, seed)
  .step_by(step).for_each(|(t, x)| 
     println!("t:{},q:{}", t, x)
  );
}