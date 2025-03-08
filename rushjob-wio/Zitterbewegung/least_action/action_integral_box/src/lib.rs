#![no_std]

extern crate alloc;
use alloc::boxed::Box;
use alloc::rc::Rc;

type T = f32;

// 微分
pub fn differential(
  h: T,                     // Δ
) -> impl Fn(
       Rc<dyn Fn(T) -> T>   // 対象関数
  ) -> Rc<dyn Fn(T) -> T> { // 導関数
  // 中心差分: (f(x+h)-f(x-h))/2h
  move |f: Rc<dyn Fn(T) -> T>|
    Rc::new(move |x|
      (f(x+h) - f(x-h)) / (2.*h)
    )
}

// 作用積分汎関数:S[x(t)]
pub fn action_integral(
  int_t: impl Fn(              // 積分演算子
           Box<dyn Fn(T) -> T> // 被積分関数
         ) -> T,
  d_t:   impl Fn(              // 微分演算子
           Rc<dyn Fn(T) -> T>  // 対象関数
         ) -> Rc<dyn Fn(T) -> T>, // 導関数
                               // ﾗｸﾞﾗﾝｼﾞｱﾝ
  lagrangian: Rc<dyn Fn(T, T) -> T>,
) -> impl Fn(
       Rc<dyn Fn(T) -> T>      // 対象関数
     ) -> T {                  // 積分結果
  move |xt: Rc<dyn Fn(T) -> T>| {
    let dxdt = d_t(Rc::clone(&xt)); // 導関数
    let l = Rc::clone(&lagrangian);
    let func_t =               // 被積分関数
      Box::new(move |t: T| l(xt(t), dxdt(t))
    );
    int_t(func_t)              // 積分
  }
}
    
// 積分
pub fn integral(
  t0  : T,     // 積分範囲(始端)
  t1  : T,     // 積分範囲(終端)
  ndiv: usize, // 分割数
) -> impl Fn(
       Box<dyn Fn(T) -> T> // 被積分関数
     ) -> T {
  // 刻み幅
  let dt = (t1 - t0) / ndiv as T;
  // 積分実行
  move |f|
    (0..ndiv)
    // 分割数毎のtの値を算出
    .map(|t| t0 + 0.5*dt + t as T * dt)
    // 被積分関数実行
    .map(f)
    // 集計
    .sum::<T>() * dt
}

