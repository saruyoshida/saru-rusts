#![no_std]

#[allow(unused_imports)]
use num_traits::Float;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use ndarray_aulait::*;
use itertools::{iproduct};

type T = f32;
type U = usize;
type I = isize;
// 格子ボルツマン法
#[allow(clippy::reversed_empty_ranges)]
pub fn vortex_lbm2dmix(
  ny       : U, // 格子行
  nx       : U, // 格子列
  viscosity: T, // 動粘性係数
  v0       : T, // 初速と流入速度 
                // 障壁
  barrier  : impl Iterator<Item=(U, U)>
             + Clone
) -> impl Iterator<Item=Array2<T>>
{
  // 緩和パラメータ 1/τ
  // vi=1/3(τ-1/2)→τ=3vi+1/2
  // Re=代表流れ*代表長さ/vi
  let omega = 1. / (3.*viscosity + 0.5);
  // D2Q9 モデルの速度ベクトル cᵢ
  let (c, ci) = (d2q9(), d2q9index());
  // 重み、逆方向インデックス
  let (w, opp) = (w(), opp());
  // v0の二乗
  let v0p2 = v0*v0;
  // すべての配列を右肩上がりの流れに初期化
  // 9方向の粒子密度
  let mut f = Array3::<T>::zeros((9,ny,nx));
 {let arr = Array2::<T>::ones((ny,nx));
  f.axis_iter_mut(Axis(0))
  .into_par_iter()
  .enumerate()
  .for_each(|(i, fi)| {
    let cv = c[(i, 1)] * v0;
    (w[i]*(&arr+3.0*cv+4.5*cv*cv-1.5*v0p2))
    .assign_to(fi);
  });
 }// arrをdrop
  // バウンスバック境界条件 -----------------
  let bounceback = 
    Array1::<((U, U, U), (U, U, U))>
          ::from_iter(
      iproduct!(
        opp.iter().enumerate().skip(1),
        barrier,
      )
      .map(|((i, oi), (y, x))| (
        (*oi,
         (y as I + ci[(*oi, 0)]) as U,
         (x as I + ci[(*oi, 1)]) as U,
        ), 
        (i, y, x)
      ))
      .filter(|((_,oy,ox), (_,y,x))|
        *y  < ny && *x  < nx &&
        *oy < ny && *ox < nx
      )
    ) 
  ;
  // 無限イテレータ ------------------------
  (0..).scan(0 , move |_, _| {
    // stream ------------------------------
    // すべてのパーティクルを進行方向に沿って
    // 一歩ずつ移動させる(pbc)：
    // fᵢ(u+cᵢΔt,t+Δt)-f*ᵢ(u,t)
    (0..2)
    .for_each(|j| {
      f.axis_iter_mut(Axis(0))
      .into_par_iter()
      .enumerate()
      .filter(|(i, _)| ci[(*i, j)] != 0)
      .for_each(|(i, fi)| 
        rolla2vm(&fi, ci[(i, j)], j)
        .assign_to(fi)
      )
    });
    // バウンスバック -----------------------
    bounceback
    .iter()
    .for_each(|(oidx, idx)| f[*oidx]=f[*idx]); 
    // 密度 ---------------------------------
    // p(u,t) = Σᵢ fᵢ(u,t)
    let p = f.sum_axis(Axis(0));
    // 速度 ---------------------------------
    // vx(u,t)=1/p[(f₁+f₅+f₈)-(f₃+f₆+f₇)]
    let vx = 
      f.axis_iter(Axis(0))
      .into_par_iter()
      .enumerate()
      .filter(|(i, _)| ci[(*i, 1)] != 0)
      .map(|(i, fi)| &fi * c[(i, 1)])
      .reduce(
        || Array2::<T>::zeros((ny,nx)),
        |sum, fc| &sum + &fc
      )
//      .unwrap()
      / &p
    ;
    // vy(u,t)=1/p[(f₂+f₅+f₆)-(f₄+f₇+f₈)]
    let vy =
      f.axis_iter(Axis(0))
      .into_par_iter()
      .enumerate()
      .filter(|(i, _)| ci[(*i, 0)] != 0)
      .map(|(i, fi)| &fi * c[(i, 0)])
      .reduce(
        || Array2::<T>::zeros((ny,nx)),
        |sum, fc| &sum + &fc
      )
//      .unwrap()
      / &p
    ;
    // 衝突 ---------------------------------
    // f*ᵢ(u,t)=fᵢ(u,t)-1/τ[fᵢ(u,t)-fᵉ⁹ᵢ(u,t)]
    // 局所平衡分布関数 
    // fᵉ⁹ᵢ(u,t)=wᵢp[1+3(cᵢ•v)+4.5(cᵢ•v)²-1.5v²]
    let v2 = &vx*&vx + &vy*&vy;

    f.axis_iter_mut(Axis(0))
    .into_par_iter()
    .enumerate()
    .for_each(|(i, fi)| {
      let cv = c[(i, 1)]*&vx + c[(i, 0)]*&vy;

      ((1.-omega) * &fi + omega * w[i] *&p * 
       (1. + 3.*&cv + 4.5*&cv*&cv -  1.5*&v2)
      ).assign_to(fi);
    });
    // 流入 ----------------------------------
    // 両端で右向きの安定した流れを強制する
    // （0、N、S成分を設定する必要はない）
    f.axis_iter_mut(Axis(0))
    .into_par_iter()
    .enumerate()
    .filter(|(i, _)| ci[(*i, 1)] != 0)
    .for_each(|(i, mut fi)|
      fi.slice_mut(s![.., 0]).fill(
        w[i] * 
        (1.+3.*v0*c[(i,1)]+4.5*v0p2-1.5*v0p2)
      )
    );
    // 渦度出力 ------------------------------
    // 巨視的な速度場のカールを計算する
    // 2で割るのは省略
    // (rot v)z= ∂vy/∂x-∂vx/∂y
    Some(
      rolla2(&vy,-1,1) - rolla2(&vy,1,1) - 
      rolla2(&vx,-1,0) + rolla2(&vx,1,0)
    )
  })
}
// D2Q9 モデルの速度ベクトル 
fn d2q9() -> Array2<T> {
  d2q9index().mapv(|q| q as T)
}
// D2Q9 ｲﾝﾃﾞｯｸｽ
fn d2q9index() -> Array2<I> {
  arr2(&[
  //  y ,  x
    [ 0 ,  0 ], // 0
    [ 0 ,  1 ], // E
    [ 1 ,  0 ], // N
    [ 0 , -1 ], // W
    [-1 ,  0 ], // S
    [ 1 ,  1 ], // NE
    [ 1 , -1 ], // NW
    [-1 , -1 ], // SW
    [-1 ,  1 ], // SE
  ])
}
// 重み w[i]
fn w() -> Array1<T> {
  Array1::<T>::from_iter(
    [4./9. as T].into_iter()
    .chain(
      [1./9.].into_iter().cycle().take(4)
    ).chain(
      [1./36.].into_iter().cycle().take(4)
    )
  )
}
// 逆方向インデックス opp[i]
fn opp() -> [U; 9] {
  [0, 3,  4,  1,  2,  7,  8,  5,  6]
}

