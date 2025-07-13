use qma::*;

type T = f32;
// いい感じのseed値を探す
fn main() {
/*
  // QMA7 第2励起(311軌道)
  let dxyz = |xt:T, yt:T, zt:T, rt:T| {
    let p = (rt-9.)/(3.*rt*(6.-rt));
    // DX(t),DY(t),DZ(t)
    (p*xt+1./xt, p*yt, p*zt)
  };
*/
  // QMA9 第2励起(310軌道)
  let dxyz = |xt:T, yt:T, zt:T, rt:T| {
    let p = (rt-9.)/(3.*rt*(6.-rt));
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt+1./zt)
  };
  let xyz0 = (3., 3., 3.);
  let cnt = 200000;
  (1..50u8).for_each(|seed| {
    let max = qma(dxyz, xyz0, 0.01, cnt, seed)
              .map(|(x, _, _, _)| x) 
              .fold(T::MIN, |a, b| 
                if a > b {a} else {b}
              );
    let min = qma(dxyz, xyz0, 0.01, cnt, seed)
              .map(|(x, _, _, _)| x) 
              .fold(T::MAX, |a, b| 
                if b < a {b} else {a}
              );
    if min < -10. && max > -3. {
      println!("seed:{},min:{},max{}", 
              seed, min, max);
    }
  });
}
     
