use chrono::{Local, Timelike};
use qma::*;

type T = f32;
fn main() {
  // 乱数シード値設定
  let time = Local::now();
  let seed = (time.nanosecond() % 256) as u8;
  // QMA1 基底(100軌道)
  let dxyz1 = |xt:T, yt:T, zt:T, rt:T| 
    // DX(t),DY(t),DZ(t)
    (-xt/rt, -yt/rt, -zt/rt)
  ;
  // QMA2 第1励起(200軌道)
  let dxyz2 = |xt:T, yt:T, zt:T, rt:T| {
    let p = -((4.-rt)/(2.*rt*(2.-rt)));
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt)
  };
  // QMA3 第1励起(211軌道)
  let dxyz3 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt)+1./xt, // DX(t)
    -yt/(2.*rt),       // DY(t)
    -zt/(2.*rt),       // DZ(t)
  );
  // QMA4 第1励起(21-1軌道)
  let dxyz4 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt),       // DX(t)
    -yt/(2.*rt)+1./yt, // DY(t)
    -zt/(2.*rt),       // DZ(t)
  );
  // QMA5 第1励起(210軌道)
  let dxyz5 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt),       // DX(t)
    -yt/(2.*rt),       // DY(t)
    -zt/(2.*rt)+1./zt, // DZ(t)
  );
  // QMA6 第2励起(300軌道)
  let dxyz6 = |xt:T, yt:T, zt:T, rt:T| {
    let p = 
      (2.*(2.*rt-9.))/
      (2.*rt.powi(3)-18.*rt.powi(2)+27.*rt)
      -1./3.*rt
    ;
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt)
  };
  // QMA7 第2励起(311軌道)
  let dxyz7 = |xt:T, yt:T, zt:T, rt:T| {
    let p = (rt-9.)/(3.*rt*(6.-rt));
    // DX(t),DY(t),DZ(t)
    (p*xt+1./xt, p*yt, p*zt)
  };
  // QMA8 第2励起(31-1軌道)
  let dxyz8 = |xt:T, yt:T, zt:T, rt:T| {
    let p = (rt-9.)/(3.*rt*(6.-rt));
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt+1./yt, p*zt)
  };
  // QMA9 第2励起(310軌道)
  let dxyz9 = |xt:T, yt:T, zt:T, rt:T| {
    let p = (rt-9.)/(3.*rt*(6.-rt));
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt+1./zt)
  };
  // QMA10 第2励起(322軌道)
  let dxyz10 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt)+(2.*xt)/(xt*xt-yt*yt), //DX(t)
    -yt/(2.*rt)-(2.*yt)/(xt*xt-yt*yt), //DY(t)
    -zt/(2.*rt),                       //DZ(t)
  );
  // QMA11 第2励起(321軌道)
  let dxyz11 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt)+1./xt, // DX(t)
    -yt/(2.*rt),       // DY(t)
    -zt/(2.*rt)+1./zt, // DZ(t)
  );
  // QMA12 第2励起(320軌道)
  let dxyz12 = |xt:T, yt:T, zt:T, rt:T| {
    let p = -(1./(2.*rt)+
              2./(2.*zt*zt-xt*xt-yt*yt))
    ;
    // DX(t),DY(t),DZ(t)
    (p*xt, p*yt, p*zt)
  };
  // QMA13 第2励起(32-1軌道)
  let dxyz13 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt),       // DX(t)
    -yt/(2.*rt)+1./yt, // DY(t)
    -zt/(2.*rt)+1./zt, // DZ(t)
  );
  // QMA14 第2励起(32-2軌道)
  let dxyz14 = |xt:T, yt:T, zt:T, rt:T| (
    -xt/(2.*rt)+1./xt, // DX(t)
    -yt/(2.*rt)+1./yt, // DY(t)
    -zt/(2.*rt),       // DZ(t)
  );
  // 結果出力
  println!("結果リスト seed:{}", seed);
  println!("QMA1");
  rlist(dxyz1 , (1.,1.,1.), seed, 5001, 1000);
  println!("QMA2");
  rlist(dxyz2 , (1.,1.,1.), seed,10001, 2000);
  println!("QMA3");
  rlist(dxyz3 , (1.,1.,1.), seed,20001, 4000);
  println!("QMA4");
  rlist(dxyz4 , (1.,1.,1.), seed,20001, 4000);
  println!("QMA5");
  rlist(dxyz5 , (1.,1.,1.), seed,20001, 4000);
  println!("QMA6");
  rlist(dxyz6 , (3.,3.,3.), seed,20001, 4000);
  println!("QMA7");
//rlist(dxyz7 , (3.,3.,3.), seed,20001, 4000);
  rlist(dxyz7 , (3.,3.,3.),seed,300000,30000);
  println!("QMA8");
  rlist(dxyz8 , (3.,3.,3.), seed,20001, 4000);
  println!("QMA9");
  rlist(dxyz9 , (3.,3.,3.), seed,20001, 4000);
  println!("QMA10");
  rlist(dxyz10, (3.,4.,5.), seed,20001, 4000);
  println!("QMA11");
  rlist(dxyz11, (3.,4.,5.), seed,20001, 4000);
  println!("QMA12");
  rlist(dxyz12, (3.,4.,5.), seed,20001, 4000);
  println!("QMA13");
  rlist(dxyz13, (3.,4.,5.), seed,20001, 4000);
  println!("QMA14");
  rlist(dxyz14, (3.,4.,5.), seed,20001, 4000);
}
// 結果リスト
fn rlist(
  dxyz: impl Fn(T, T, T, T) 
             -> (T, T, T) +'static,
  xyz0: (T, T, T),
  seed: u8,    // 乱数シード
  cnt : usize, // 回数
  step: usize, // 観測ステップ
) {
  qma(dxyz, xyz0, 0.01, cnt,seed)
  .step_by(step).for_each(|(x, y, z, _r)| 
     println!("x:{},y:{},z:{}", x,y,z)
  );
}