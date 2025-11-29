#![no_std]

#[allow(unused_imports)]
use num_traits::Float;
use ndarray::prelude::*;
use ndarray_aulait::*;
use itertools::{iproduct};

type T = f32;
type U = usize;
type I = isize;
// 格子ボルツマン法
#[allow(clippy::reversed_empty_ranges)]
pub fn vortex_lbm3d(
  ny       : U, // 格子行
  nx       : U, // 格子列
  nz       : U, // 格子奥
  viscosity: T, // 液体粘度
  v0       : T, // 初速と流入速度 
                // 障壁
  barrier  : impl Iterator<Item=(U, U, U)>
             + Clone
)-> impl Iterator
      <Item=(Array3<T>, Array3<T>, Array3<T>)>
{
  // 緩和パラメータ 1/τ
  // vi=1/3(τ-1/2)→τ=3vi+1/2
  // Re=代表流れ*代表長さ/vi
  let omega = 1. / (3.*viscosity + 0.5);
  // D3Q19 モデルの速度ベクトル cᵢ
  let (c, ci) = (d3q19(), d3q19index());
  // 重み、逆方向インデックス
  let (w, opp) = (w(), opp());
  // v0の二乗
  let v0p2 = v0*v0;
  // すべての配列を右肩上がりの流れに初期化
  // 19方向の粒子密度
  let mut f = 
    Array4::<T>::zeros((19, nz, ny, nx))
  ;
 {let arr = Array3::<T>::ones((nz, ny, nx));
  (0..19)
  .for_each(|i| {
    let cv = c[(i, 2)] * v0;
    f.slice_mut(s![i, .., .., ..]).assign(&(
      w[i] * 
      (&arr + 3.0*cv + 4.5*cv*cv - 1.5*v0p2)
    ))
  });
 }// arrをdrop
  // バウンスバック境界条件 -----------------
  let bounceback = 
    Array1::<((U, U, U, U), (U, U, U, U))>
          ::from_iter(
      iproduct!(
        opp.iter().enumerate().skip(1),
        barrier,
      )
      .map(|((i, oi), (y, x, z))| (
        (*oi,
         (z as I + ci[(*oi, 0)]) as U,
         (y as I + ci[(*oi, 1)]) as U,
         (x as I + ci[(*oi, 2)]) as U,
        ), 
        (i, y, x, z)
      ))
      .filter(|((_,oy,ox,oz), (_,y,x,z))|
        *z  < nz && *y  < ny && *x  < nx &&
        *oz < nz && *oy < ny && *ox < nx
      )
    ) 
  ;
  // 無限イテレータ ------------------------
  (0..).scan(0 , move |_, _| {
    // stream ------------------------------
    // すべてのパーティクルを進行方向に沿って
    // 一歩ずつ移動させる(pbc)：
    // fᵢ(u+cᵢΔt,t+Δt)-f*ᵢ(u,t)
    iproduct!(1..19, 0..3)
    .filter(|(i, j)| ci[(*i, *j)] != 0)
    .for_each(|(i, j)| {
      rolla3v(
        f.slice(s![i, .., .., ..]),
        ci[(i, j)],
        j,
      )
      .assign_to(f.slice_mut(s![i,..,..,..]));
    });
    // バウンスバック -----------------------
    bounceback
    .iter()
    .for_each(|(oidx, idx)| f[*oidx]=f[*idx]); 
    // 密度 ---------------------------------
    // p(u,t) = Σᵢ fᵢ(u,t)
    let p = f.sum_axis(Axis(0));
    // 速度 ---------------------------------
    // vx(x,t)=1/p(Σᵢ cᵢxfᵢ(u,t))
    let vx =
      (0..19)
      .filter(|i| ci[(*i, 2)] != 0)
      .map(|i| 
        &f.slice(s![i,..,..,..]) * c[(i, 2)]
      )
      .reduce(|sum, fc| &sum + &fc).unwrap()
      / &p
    ;
    // vy(u,t)=1/p(Σᵢ cᵢyfᵢ(u,t))
    let vy = 
      (0..19)
      .filter(|i| ci[(*i, 1)] != 0)
      .map(|i| 
        &f.slice(s![i,..,..,..]) * c[(i, 1)]
      )
      .reduce(|sum, fc| &sum + &fc).unwrap()
      / &p
    ;
    // vz(u,t)=1/p(Σᵢ cᵢzfᵢ(u,t))
    let vz =
      (0..19)
      .filter(|i| ci[(*i, 0)] != 0)
      .map(|i| 
        &f.slice(s![i,..,..,..]) * c[(i, 0)]
      )
      .reduce(|sum, fc| &sum + &fc).unwrap()
      / &p
    ;
    // 衝突 ---------------------------------
    // f*ᵢ(u,t)=fᵢ(u,t)-1/τ[fᵢ(u,t)-fᵉ⁹ᵢ(u,t)]
    // 局所平衡分布関数 
    // fᵉ⁹ᵢ(u,t)=wᵢp[1+3(cᵢ•v)+4.5(cᵢ•v)²-1.5v²]
    let v2 = &vx*&vx + &vy*&vy + &vz*&vz;
    (0..19)
    .for_each(|i| {
      let cv = &vx*c[(i, 2)] + 
               &vy*c[(i, 1)] +
               &vz*c[(i, 0)];

      ((1.-omega) * &f.slice(s![i,..,..,..]) +
       omega * w[i] * &p * 
       (1. + 3.*&cv + 4.5*&cv*&cv - 1.5*&v2)
      )
      .assign_to(f.slice_mut(s![i,..,..,..]));
    });
    // 流入 ----------------------------------
    // 両端で右向きの安定した流れを強制する
    (0..19)
    .filter(|i| ci[(*i, 2)] != 0)
    .for_each(|i|
      f.slice_mut(s![i, .., .., 0]).fill(
        w[i] * 
        (1.+3.*v0*c[(i,2)]+4.5*v0p2-1.5*v0p2)
      )
    );
    // 渦度出力 ------------------------------
    // 巨視的な速度場のカールを計算する
    Some((
      // (rot v)y= ∂vz/∂x-∂vx/∂z
      rolla3(&vz,-1,1) - rolla3(&vz,1,1) - 
      rolla3(&vx,-1,2) + rolla3(&vx,1,2),
      // (rot v)x= ∂vy/∂z-∂vz/∂y
      rolla3(&vy,-1,1) - rolla3(&vy,1,2) - 
      rolla3(&vz,-1,0) + rolla3(&vz,1,0),
      // (rot v)z= ∂vy/∂x-∂vx/∂y
      rolla3(&vy,-1,2) - rolla3(&vy,1,1) - 
      rolla3(&vx,-1,0) + rolla3(&vx,1,0),
    ))
  })
}
// D3Q19 モデルの速度ベクトル 
fn d3q19() -> Array2<T> {
  d3q19index().mapv(|q| q as T)
}
// D3Q19 ｲﾝﾃﾞｯｸｽ
fn d3q19index() -> Array2<I> {
  arr2(&[
  //  z,   y ,  x 
    [ 0 ,  0 ,  0 ], // 0
    [ 0 ,  0 ,  1 ], // 1  W
    [ 0 ,  0 , -1 ], // 2  E
    [ 1 ,  0 ,  0 ], // 3  B
    [-1 ,  0 ,  0 ], // 4  F
    [ 0 ,  1 ,  0 ], // 5  N
    [ 0 , -1 ,  0 ], // 6  S
    [-1 ,  0 , -1 ], // 7  EB
    [-1 ,  0 ,  1 ], // 8  WB
    [ 1 ,  0 , -1 ], // 9  EF
    [ 1 ,  0 ,  1 ], // 10 WF
    [ 0 ,  1 ,  1 ], // 11 NE
    [ 0 ,  1 , -1 ], // 12 NW
    [ 0 , -1 ,  1 ], // 13 SE
    [ 0 , -1 , -1 ], // 14 SW
    [-1 , -1 ,  0 ], // 15 SB
    [ 1 ,  1 ,  0 ], // 16 NF
    [ 1 , -1 ,  0 ], // 17 SF
    [-1 ,  1 ,  0 ], // 18 NB
  ])
}
// 重み w[i]
fn w() -> Array1<T> {
  Array1::<T>::from_iter(
    [1./3. as T].into_iter()
    .chain(
      [1./18.].into_iter().cycle().take(6)
    ).chain(
      [1./36.].into_iter().cycle().take(12)
    )
  )
}
// 逆方向インデックス opp[i]
fn opp() -> [U; 19] {
  [0, 2,  1,  4,  3,  6,  5,  8,  7, 10, 9,
     12, 11, 14, 13, 16, 15, 18, 17]
}

