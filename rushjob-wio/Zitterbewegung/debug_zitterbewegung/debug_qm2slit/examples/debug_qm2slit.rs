use qms::*;

type T = f32;
// 電子の自由運動
fn main() {
  // 乱数シード値設定
  let seed = 43u8;
  // スリット設定
  let a: T = 0.3;       // スリット巾
  let s: T = 0.3;       // ｽﾘｯﾄ間隔半値
  let s2   = s.powi(2); // s^2
  let s4   = s.powi(4); // s^4
  // 二重スリット(本)
  let d0   = move |yt:T, t: T| {
    let t2a2 = t*t+a*a;
    let ts   = t*s;
    // (2(t-a)/(t^2+a^2))Y(t) +
    // (ts/t^2+a^2)*tan(ts/t^2+a^2*Y(t))
    (2.*(t-a)) / t2a2 * yt +
    ts / t2a2 * (ts / t2a2 * yt).tan()
  };
  // 二重スリット(Excelの内容)
  let d1 = move |yt:T, t: T|
    // t-2s^2 / t^2+4s^4 Y(t)
    (t-2.*s2) / (t*t+4.*s4) * yt +
    ((  // sinh(4s^2sY(t) / 4s^4+t^2) *
        //     (t-2s^2    / t^2+4s^4)
      ((4.*s2*s*yt) / (4.*s4+t*t)).sinh() *
      ((t-2.*s2)    / (t*t+4.*s4))
      - // sin(2tsY(t) / 4s^4+t^2) *
        //    (t+2s^2  / t^2+4s^4)
      ((2.*t*s*yt)  / (4.*s4+t*t)).sin()  *
      ((t+2.*s2)    / (t*t+4.*s4))
     ) /
     (  // cosh(4s^2sY(t) / 4s^4+t^2) +
        //  cos(2tsY(t)   / 4s^4+t^2)
      ((4.*s2*s*yt) / (4.*s4+t*t)).cosh() +
      ((2.*t*s*yt)  / (4.*s4+t*t)).cos()
     )
    ) * s
  ;
  // 単スリット
  let d2   = move |yt:T, t: T|
    // t-2a^2 / t^2+4a^4 * Y(t)
    (t-2.*s2) / (t*t+4.*s4) * yt
  ;
  // 結果出力
  println!("二重スリット（本）");
  result_list(d0, seed, 300, 60);
  println!("二重スリット（Excel）");
  result_list(d1, seed, 300, 60);
  println!("単スリット");
  result_list(d2, seed, 300, 60);
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