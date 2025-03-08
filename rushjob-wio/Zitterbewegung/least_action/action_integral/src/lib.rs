#![no_std]
type T = f32;

// 微分
pub fn differential(
  h: T,               // Δ
  f: impl Fn(T) -> T, // 対象関数
) -> impl Fn(T) -> T {
  // 中心差分: (f(x+h)-f(x-h))/2h
  move |x| (f(x+h) - f(x-h)) / (2.*h)
}

// 作用
pub fn action(           // ﾗｸﾞﾗﾝｼﾞｱﾝ                    
  lagrangian: impl Fn(T, T) -> T, 
  xt  : impl Fn(T) -> T, // 対象関数
  dxdt: impl Fn(T) -> T, // 対象関数微分
) -> impl Fn(T) -> T {
  // L(x(t), x'(t))
  move |t| lagrangian(xt(t), dxdt(t))
}

// 積分
pub fn integral(
  t0  : T,            // 積分範囲(始端)
  t1  : T,            // 積分範囲(終端)
  ndiv: usize,        // 分割数
  f: impl Fn(T) -> T, // 被積分関数
) -> T {
  // 刻み幅
  let dt = (t1 - t0) / ndiv as T;
  // 積分実行
  (0..ndiv)
  // 分割数毎のtの値を算出
  .map(|t| t0 + 0.5*dt + t as T * dt)
  // 被積分関数実行
  .map(f)
  // 集計
  .sum::<T>() * dt
}

