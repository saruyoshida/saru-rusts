//※ndarray自主練①
#![no_std]

#[allow(unused_imports)]
use num_traits::Float;
use ndarray::{s, Array2};

type T = f32;
// 2次元キャビティ流れ
#[allow(clippy::too_many_arguments)]
#[allow(clippy::reversed_empty_ranges)]
pub fn cavityflow2d(
  nx : usize,  // 格子列
  ny : usize,  // 格子行
  nit: usize,  // 圧力計算回数
  rho: T,      // 密度
  nu : T,      // 動粘性係数
  dt : T,      // Δt
  dx : T,      // Δx
  dy : T,      // Δy
)-> impl Iterator<Item=(Array2<T>, Array2<T>)>
{ // Array 作成
  let mut u = Array2::<T>::zeros((nx, ny));
  let mut v = Array2::<T>::zeros((nx, ny));
  let mut p = Array2::<T>::zeros((nx, ny));
  let mut b = Array2::<T>::zeros((nx, ny));
  let mut un = u.clone();
  let mut vn = v.clone();
  // =======================================
  // 無限イテレータ
  (0..).scan(0, move |_, _| {
    core::mem::swap(&mut u, &mut un);
    core::mem::swap(&mut v, &mut vn);
        
    b = build_up_b(
      b.clone(), rho, dt, &u, &v, dx, dy
    );
    p = pressure_poisson(
      p.clone(), dx, dy, &b, nit
    );
    // uⁿ⁺¹ᵢ,ⱼ = 
    u.slice_mut(s![1..-1, 1..-1]).assign(&(
      // uⁿᵢ,ⱼ - uⁿᵢ,ⱼΔt/Δx(uⁿᵢ,ⱼ - uⁿᵢ₋₁,ⱼ)
      &un.slice(s![1..-1, 1..-1]) -
      &un.slice(s![1..-1, 1..-1]) *
      dt/dx * (
        &un.slice(s![1..-1, 1..-1]) -
        &un.slice(s![1..-1,  ..-2]) 
      ) -
      // vⁿᵢ,ⱼΔt/Δy(uⁿᵢ,ⱼ - uⁿᵢ,ⱼ₋₁)
      &vn.slice(s![1..-1, 1..-1]) *
      dt/dy * (
        &un.slice(s![1..-1, 1..-1]) -
        &un.slice(s![ ..-2, 1..-1])
      ) -
      // Δt/ρ2Δx(pⁿᵢ₊₁,ⱼ + pⁿᵢ₋₁,ⱼ)
      dt/(2.*rho*dx) * (
        &p.slice(s![1..-1, 2.. ]) -
        &p.slice(s![1..-1, ..-2])
      ) +
      // μ
      nu*( 
       // Δt/Δx²(uⁿᵢ₊₁,ⱼ-2uⁿᵢ,ⱼ+uⁿᵢ₋₁,ⱼ)
       dt/dx.powi(2) *
       (&un.slice(s![1..-1, 2..]) -
        2. * &un.slice(s![1..-1, 1..-1]) +
              un.slice(s![1..-1,  ..-2])
       ) +
       // Δt/Δy²(uⁿᵢ,ⱼ₊₁-2uⁿᵢ,ⱼ+uⁿᵢ,ⱼ₋₁)
       dt/dy.powi(2) *
       (&un.slice(s![2..,  1..-1]) -
        2. * &un.slice(s![1..-1, 1..-1]) +
              un.slice(s![ ..-2, 1..-1])
       )
      )
    ));
    // vⁿ⁺¹ᵢ,ⱼ = 
    v.slice_mut(s![1..-1,1..-1]).assign(&(
      // vⁿᵢ,ⱼ - uⁿᵢ,ⱼΔt/Δx(vⁿᵢ,ⱼ - vⁿᵢ₋₁,ⱼ)
      &vn.slice(s![1..-1, 1..-1]) -
      &un.slice(s![1..-1, 1..-1]) *
      dt/dx * (
        &vn.slice(s![1..-1, 1..-1]) -
        &vn.slice(s![1..-1,  ..-2])
      ) -
      // vⁿᵢ,ⱼ - vⁿᵢ,ⱼΔt/Δy(vⁿᵢ,ⱼ - vⁿᵢ,ⱼ₋₁)
      &vn.slice(s![1..-1, 1..-1]) *
      dt/dy * (
        &vn.slice(s![1..-1 ,1..-1]) -
        &vn.slice(s![ ..-2, 1..-1])
      ) -
      // Δt/ρ2Δy(pⁿᵢ,ⱼ₊₁ + pⁿᵢ,ⱼ₋₁)
      dt/(2.*rho*dy)* (
        &p.slice(s![2.. , 1..-1]) -
        &p.slice(s![..-2, 1..-1])
      ) +
      // μ
      nu*(
        // Δt/Δx²(vⁿᵢ₊₁,ⱼ-2vⁿᵢ,ⱼ+vⁿᵢ₋₁,ⱼ)
        dt/dx.powi(2) * (
          &vn.slice(s![1..-1, 2..])-
          2.* &vn.slice(s![1..-1, 1..-1]) + 
               vn.slice(s![1..-1,  ..-2])
        ) +
        // Δt/Δy²(vⁿᵢ,ⱼ₊₁-2vⁿᵢ,ⱼ+vⁿᵢ,ⱼ₋₁)
        dt/dy.powi(2) * (
          &vn.slice(s![2.., 1..-1])-
          2.* &vn.slice(s![1..-1, 1..-1]) + 
               vn.slice(s![..-2 , 1..-1])
        )
      )
    ));
 
    u.slice_mut(s![0 , ..]).fill(0.);
    u.slice_mut(s![.., 0 ]).fill(0.);
    u.slice_mut(s![.., -1]).fill(0.);
    // set velocity on cavity lid equal to 1
    u.slice_mut(s![-1, ..]).fill(1.);   
    v.slice_mut(s![0 , ..]).fill(0.);
    v.slice_mut(s![-1, ..]).fill(0.);
    v.slice_mut(s![..,  0]).fill(0.);
    v.slice_mut(s![.., -1]).fill(0.);
    
    Some((u.clone(), v.clone()))
  })
}
// pⁿ⁺¹ᵢ,ⱼ
#[allow(clippy::reversed_empty_ranges)]
pub fn pressure_poisson(
  mut p: Array2<T>,
  dx   : T,
  dy   : T,
  b    : &Array2<T>,
  nit  : usize,
) -> Array2<T> {
  let mut pn = p.clone();
  (0..nit).for_each(|_| {
    core::mem::swap(&mut p, &mut pn);
    // pⁿ⁺¹ᵢ,ⱼ = 
    p.slice_mut(s![1..-1, 1..-1]).assign(&(
      (// (pⁿ⁺¹ᵢ₊₁,ⱼ + pⁿ⁺¹ᵢ₋₁,ⱼ)Δy²
        (&pn.slice(s![1..-1, 2.. ]) +
         &pn.slice(s![1..-1, ..-2])
        ) * dy*dy +
        // (pⁿ⁺¹ᵢ,ⱼ₊₁ + pⁿ⁺¹ᵢ,ⱼ₋₁)Δx²
        (&pn.slice(s![2.. , 1..-1]) +
         &pn.slice(s![..-2, 1..-1])
        ) * dx*dx
      ) /
      // 2(Δx² + Δy²)
      (2.* (dx*dx + dy*dy)) -
      // bⁿᵢ,ⱼΔx²Δy²/2(Δx²+Δy²)
        (dx*dx) * (dy*dy) /
        (2.* (dx*dx + dy*dy)) * 
        &b.slice(s![1..-1, 1..-1])
      )
    );
  });
  // dp/dx = 0 at x = 2
  let dpdxx2 = p.slice(s![.., -2]).to_owned();
  p.slice_mut(s![.., -1]).assign(&dpdxx2);
  // dp/dy = 0 at y = 0
  let dpdyy0 = p.slice(s![1 , ..]).to_owned();
  p.slice_mut(s![0 , ..]).assign(&dpdyy0);
  // dp/dx = 0 at x = 0
  let dpdxx0 = p.slice(s![.., 1 ]).to_owned();
  p.slice_mut(s![.., 0 ]).assign(&dpdxx0);
  // p = 0 at y = 2
  p.slice_mut(s![-1, ..]).fill(0.); 

  p
}
// ∂²p/∂x² + ∂²p/∂y² = b(x, y)
// bⁿᵢ,ⱼ
#[allow(clippy::reversed_empty_ranges)]
pub fn build_up_b(
  mut b: Array2<T>,
  rho  : T,
  dt   : T,
  u    : &Array2<T>,
  v    : &Array2<T>,
  dx   : T,
  dy   : T,
) -> Array2<T> {
  b.slice_mut(s![1..-1, 1..-1]).assign(&(
    rho * (
      1./dt * // ∂/∂t
      ( // ∂u/∂x
        (&u.slice(s![1..-1, 2.. ]) -
         &u.slice(s![1..-1, ..-2])
        ) / (2.*dx) +
        // ∂v/∂y
        (&v.slice(s![2.. , 1..-1]) - 
         &v.slice(s![..-2, 1..-1])
        ) / (2.*dy)
      ) - 
      ( // ∂u/∂x * ∂u/∂x
        (&u.slice(s![1..-1, 2.. ]) -
         &u.slice(s![1..-1, ..-2])
        ) / (2.*dx)
      ).mapv(|a| a.powi(2)) -
      2. *
      ( // ∂u/∂y * ∂v/∂x
        (&u.slice(s![2.. , 1..-1])- 
         &u.slice(s![..-2, 1..-1])
        ) / (2.*dy) *
        (&v.slice(s![1..-1, 2.. ]) -
         &v.slice(s![1..-1, ..-2])
        ) / (2.*dx)
      ) -
      ( // ∂v/∂y * ∂v/∂y
        (&v.slice(s![2.. , 1..-1]) -
         &v.slice(s![..-2, 1..-1])
        ) / (2.*dy)
      ).mapv(|a| a.powi(2))
    )
  ));
 
  b
}
