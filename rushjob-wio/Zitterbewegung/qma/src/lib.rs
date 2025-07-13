#![no_std]
use rand::prelude::*;
#[allow(unused_imports)]
use num_traits::Float;

type T = f32;
pub fn qma(
             // X(t),Y(t),Z(t),R(t)
  dxyz: impl Fn(T, T, T, T)
             // DX(t),DY(t),DZ(t)
             -> (T, T, T)+'static,
  xyz0: (T, T, T), // 初期値(x, y, z)
  dt  : T,         // 時間増分
  cnt : usize,     // 回数
  seed: u8,        // 乱数シード
                   // 結果位置(x, y, z, r)
) -> impl Iterator<Item=(T, T, T, T)> { 
  // 正規分布乱数A(t)
  let mut rng = StdRng::from_seed([seed; 32]);
  let mut at = move || {
    let arr: [T; 12] = rng.gen();
    arr.iter().sum::<T>() - 6.
  };
  // 結果出力
  (0..cnt)
  .scan(xyz0, move |(xt, yt, zt) ,_| {
     let (x, y, z) = (*xt, *yt, *zt);
     // √X(t)^2+Y(t)^2+Z(t)^2
     let rt = (x*x + y*y + z*z).sqrt();
     // 確率微分方程式
     let (dx, dy, dz) = dxyz(*xt,*yt,*zt,rt);                             
     *xt += dx*dt+at()*dt.sqrt();
     *yt += dy*dt+at()*dt.sqrt();
     *zt += dz*dt+at()*dt.sqrt();
     Some((x, y, z, rt))
  })
}

