use action_integral_box::*;
use std::rc::Rc;

type T = f32;

fn main() {
  // 試行関数の作用積分汎関数
  let action = create_trial_action();
  // 結果リスト
  action_rows(Rc::clone(&action)).for_each(
    |(st, k)| println!("{},{}", k, st)
  );
  // 最大点
  println!("max:{:?}", 
    action_rows(Rc::clone(&action))
    .fold((T::MIN, T::MIN),
      |b, c| if c.0 > b.0 {c} else {b}
    )
  ); 
  // 最小点
  println!("min:{:?}", 
    action_rows(Rc::clone(&action))
    .fold((T::MAX, T::MAX),
      |b, c| if c.0 < b.0 {c} else {b}
    )
  ); 
  // 停留点
                                   // 検証位置
  action_rows(Rc::clone(&action)).skip(1).zip(
                                   // １つ前
  action_rows(Rc::clone(&action)).zip( 
  action_rows(action).skip(2)))    // １つ後
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

// 変分範囲の写像
fn action_rows(
  trial_action: Rc<dyn Fn(T) -> T>
) -> impl Iterator<Item=(T, T)>  {
  (15..28)
  .map(|k| k as T * 0.1)
  .map(move |k| (
     trial_action(k),
     k,
  ))
}

// 振動子のﾗｸﾞﾗﾝｼﾞｱﾝ
pub fn oscillator(
  m: T,    // 質量
  k: T,    // バネ定数
) -> Rc<dyn Fn(T, T) -> T> {
  // 1/2Mv^2 - 1/2Kx^2
  Rc::new(move |x, v| 0.5 * (m*v*v - k*x*x))
}

fn create_trial_action() 
  -> Rc<dyn Fn(T) -> T>  {
  let m  = 1.0;      // 質量
  let w  = 2.5;      // √バネ定数
  let t0 = 0.0;      // 積分範囲(始端)
  let t1 = 1.0;      // 積分範囲(終端)
  let ndiv = 200;    // 分割数
  let dt   = 0.0001; // Δt
  // 積分演算子
  let int_t =  integral(t0, t1, ndiv,);
  // 微分演算子
  let d_t = differential(dt);
  // ﾗｸﾞﾗﾝｼﾞｱﾝ
  let lagrangian = oscillator(m, w*w);
  // 作用積分汎関数:S[x(t)]
  let action = action_integral(
    int_t, d_t, lagrangian
  );
  // 対象関数x(t):sin(kt)/sin(kt1)
  Rc::new(
    move |k: T| {
      let xt = Rc::new(move |t: T|  
                 (k*t).sin() / (k*t1).sin()
               );
      action(xt)
    }
  )
}

