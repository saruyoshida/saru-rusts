use action_integral::*;

type T = f32;

fn main() {
  // 結果リスト
  action_rows().for_each(|(st, k)| 
    println!("{},{}", k, st)
  );
  // 最大点
  println!("max:{:?}", 
    action_rows().fold((T::MIN, T::MIN),
      |b, c| if c.0 > b.0 {c} else {b}
    )
  ); 
  // 最小点
  println!("min:{:?}", 
    action_rows().fold((T::MAX, T::MAX),
      |b, c| if c.0 < b.0 {c} else {b}
    )
  ); 
  // 停留点
  action_rows().skip(1).zip( // 検証位置
  action_rows().zip(         // １つ前
  action_rows().skip(2)))    // １つ後
  .filter_map(|(c,(b, f))|
     if (c.0 >= b.0 && c.0 >= f.0) ||
        (c.0 <= b.0 && c.0 <= f.0) {
       Some(c)
     } else {
       None
     }
  )
  .for_each(|c| println!("停留点:{:?}", c));   
}

// 振動子のﾗｸﾞﾗﾝｼﾞｱﾝ
pub fn oscillator(
  m: T,    // 質量
  k: T,    // バネ定数
) -> impl Fn(T, T) -> T {
  // 1/2Mv^2 - 1/2Kx^2
  move |x, v| 0.5 * (m*v*v - k*x*x)
}

// 試行関数の変分
fn action_rows() 
  -> impl Iterator<Item=(T, T)>  {
  let m  = 1.0;      // 質量
  let w  = 2.5;      // √バネ定数
  let t0 = 0.0;      // 積分範囲(始端)
  let t1 = 1.0;      // 積分範囲(終端)
  let ndiv = 200;    // 分割数
  let dt   = 0.0001; // Δt

  // 変分範囲の写像
  (15..28)
  .map(|k| k as T * 0.1)
  .map(move |k| (
     // 対象関数x(t):sin(kt)/sin(kt1)
     move |t: T| (k*t).sin() / (k*t1).sin(),
     k,
  ))
  .map(move |(xt, k)| (
     // 作用:L(x(t), x'(t))を関数合成
     action(
       oscillator(m, w*w),   // ﾗｸﾞﾗﾝｼﾞｱﾝ
       xt,                   // 対象関数
       differential(dt, xt), // 対象関数微分
     ),
     k,
  ))
  .map(move |(action, k)| (
     // 積分:∫t0→t1 L(x(t), x'(t))
     integral(
       t0,                 // 積分範囲(始端)
       t1,                 // 積分範囲(終端)
       ndiv,               // 分割数
       action,             // 作用
     ), 
     k,
  ))
}

