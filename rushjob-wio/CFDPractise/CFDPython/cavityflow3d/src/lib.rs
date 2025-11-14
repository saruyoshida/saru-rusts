//※ndarray自主練③
#![no_std]

#[allow(unused_imports)]
use num_traits::Float;
use ndarray::{s, Array3};

type T  = f32;
// 3次元キャビティ流れ
#[allow(clippy::too_many_arguments)]
#[allow(clippy::reversed_empty_ranges)]
pub fn cavityflow3d(
  nx : usize, // 格子x
  ny : usize, // 格子y
  nz : usize, // 格子z
  lm : usize, // 収束判定回数
  re : T,     // ﾚｲﾉﾙｽﾞ数
  dt : T,     // Δt
  dx : T,     // Δx
  dy : T,     // Δy
  dz : T,     // Δz
  eps: T,     // 誤差閾値
)-> impl Iterator
      <Item=(Array3<T>, Array3<T>, Array3<T>)>
{
  // パラメータ設定
  let xdh = 0.5*(nx as T -1.);
  let ydh = 0.5*(ny as T -1.);
  let zdh = 0.5*(nz as T -1.);

  let dx2 = dx*dx; let idx2 = 1./dx2;
  let dy2 = dy*dy; let idy2 = 1./dy2;
  let dz2 = dz*dz; let idz2 = 1./dz2;

  let idxyz = 
    0.5 * dx2 * dy2 * dz2 / 
    (dy2 * dz2 + dx2 * dz2 + dx2 * dy2)
  ;
  let td  = 1./dt;
  let ire = 1./re;
  // Array作成
  let mut u = Array3::<T>::zeros((nx,ny,nz));
  let mut v = Array3::<T>::zeros((nx,ny,nz));
  let mut w = Array3::<T>::zeros((nx,ny,nz));
  let mut ut = u.clone();
  let mut vt = v.clone();
  let mut wt = w.clone();
  let mut p  = u.clone();
  let mut q  = u.clone();
  // スライス設定
  let sijk  = s![1..-1, 1..-1, 1..-1];
  let s2_jk = s![2..  , 1..-1, 1..-1];
  let s_2jk = s![ ..-2, 1..-1, 1..-1];
  let si2_k = s![1..-1, 2..  , 1..-1];
  let si_2k = s![1..-1,  ..-2, 1..-1];
  let sij2_ = s![1..-1, 1..-1, 2..  ];
  let sij_2 = s![1..-1, 1..-1,  ..-2];
  // =========================================
  // 無限イテレータ
  (0..).scan(0, move |_, _| {
    // 境界条件初期化 ------------------------
    boundary_init(
      &mut u, &mut ut, 
      &mut v, &mut vt,
      &mut w, &mut wt,
    );
    // 仮速度更新 ----------------------------
    let u_c = &u.slice(sijk);
    let v_c = &v.slice(sijk);
    let w_c = &w.slice(sijk);

    // V⃰ ⃰ -V/Δt+(V•▽)V=-1/Re△V
    // ▽•V=0
    // V⃰ ⃰ =V-1/Re△VΔt

    // u*=u+Δt{{-u∂u/∂x-v∂u/∂y-w∂u/∂z +
    //    1/Re(∂²u/∂x²+∂²u/∂y²+∂²u/∂z²)}
    
    // u∂u/∂x+v∂u/∂y+w∂u/∂z
    let rx = u_c*(&u.slice(s2_jk) - 
                  &u.slice(s_2jk))*xdh +
             v_c*(&u.slice(si2_k) -
                  &u.slice(si_2k))*ydh + 
             w_c*(&u.slice(sij2_) -
                  &u.slice(sij_2))*zdh
    ;
    let ry = u_c*(&v.slice(s2_jk) - 
                  &v.slice(s_2jk))*xdh + 
             v_c*(&v.slice(si2_k) -
                  &v.slice(si_2k))*ydh + 
             w_c*(&v.slice(sij2_) -
                  &v.slice(sij_2))*zdh
    ;
    let rz = u_c*(&w.slice(s2_jk) -
                  &w.slice(s_2jk))*xdh + 
             v_c*(&w.slice(si2_k) - 
                  &w.slice(si_2k))*ydh +
             w_c*(&w.slice(sij2_) -
                  &w.slice(sij_2))*zdh
    ;
    // 2次導関数（拡散項）
    let vx = (&u.slice(s2_jk) - 2.*u_c +
               u.slice(s_2jk))*idx2 +
             (&u.slice(si2_k) - 2.*u_c + 
               u.slice(si_2k))*idy2 + 
             (&u.slice(sij2_) - 2.*u_c +
               u.slice(sij_2))*idz2
    ;
    let vy = (&v.slice(s2_jk) - 2.*v_c + 
               v.slice(s_2jk))*idx2 + 
             (&v.slice(si2_k) - 2.*v_c +
               v.slice(si_2k))*idy2 + 
             (&v.slice(sij2_) - 2.*v_c +
               v.slice(sij_2))*idz2
    ;
    let vz = (&w.slice(s2_jk) - 2.*w_c +
               w.slice(s_2jk))*idx2 + 
             (&w.slice(si2_k) - 2.*w_c +
               w.slice(si_2k))*idy2 + 
             (&w.slice(sij2_) - 2.*w_c +
               w.slice(sij_2))*idz2
    ;
    // 仮速度更新
    ut.slice_mut(sijk).assign(&(
      u_c + dt*(-&rx + &vx*ire)
    ));
    vt.slice_mut(sijk).assign(&(
      v_c + dt*(-&ry + &vy*ire)
    ));
    wt.slice_mut(sijk).assign(&(
      w_c + dt*(-&rz + &vz*ire)
    ));
    // 圧力計算 -----------------------------
    // Poisson方程式の右辺 q を計算
    q.slice_mut(sijk).assign(&(
      ((&ut.slice(s2_jk)-&ut.slice(s_2jk))*xdh
       +
       (&vt.slice(si2_k)-&vt.slice(si_2k))*ydh
       +
       (&wt.slice(sij2_)-&wt.slice(sij_2))*zdh
      ) * td
    ));
    // Poisson方程式の反復解法
    for _ in 0..lm {
      // 境界条件の更新（内側値をコピー）
      boundary_update(&mut p);
      // ラプラシアン項の計算
      let rhs = 
        (&p.slice(s2_jk)+&p.slice(s_2jk))*
        idx2 +
        (&p.slice(si2_k)+&p.slice(si_2k))*
        idy2 +
        (&p.slice(sij2_)+&p.slice(sij_2))*
        idz2
      ;
      // 修正量と誤差評価
      let uli = (&rhs - &q.slice(sijk))*idxyz-
                 p.slice(sijk)
      ;
      *&mut p.slice_mut(sijk) += &uli;
      // 残差（収束判定）
      if (&uli*&uli).sum() <= eps*nz as T {
        break;
      }
    }
    // 速度の更新 ----------------------------
    // u^{n+1} = u* - dt * dp/dx
    u.slice_mut(sijk).assign(&(
      &ut.slice(sijk) - 
      dt*(&p.slice(s2_jk)-&p.slice(s_2jk))*xdh
    ));
    v.slice_mut(sijk).assign(&(
      &vt.slice(sijk) - 
      dt*(&p.slice(si2_k)-&p.slice(si_2k))*ydh
    ));
    w.slice_mut(sijk).assign(&(
      &wt.slice(sijk) - 
      dt*(&p.slice(sij2_)-&p.slice(sij_2))*zdh
    ));
    // 出力
    Some((u.clone(), v.clone(), w.clone()))
  })
  // =========================================
}
// 境界条件初期化
fn boundary_init(
  u : &mut Array3::<T>,
  ut: &mut Array3::<T>,
  v : &mut Array3::<T>,
  vt: &mut Array3::<T>,
  w : &mut Array3::<T>,
  wt: &mut Array3::<T>,
) {
  u .slice_mut(s![.., ..,  0]).fill(0.);
  ut.slice_mut(s![.., ..,  0]).fill(0.);
  v .slice_mut(s![.., ..,  0]).fill(0.);
  vt.slice_mut(s![.., ..,  0]).fill(0.);
  w .slice_mut(s![.., ..,  0]).fill(0.);
  wt.slice_mut(s![.., ..,  0]).fill(0.);

  u .slice_mut(s![.., .., -1]).fill(0.95);
  ut.slice_mut(s![.., .., -1]).fill(0.95);
  v .slice_mut(s![.., .., -1]).fill(0.15);
  vt.slice_mut(s![.., .., -1]).fill(0.15);
  w .slice_mut(s![.., .., -1]).fill(0.);
  wt.slice_mut(s![.., .., -1]).fill(0.);

  u .slice_mut(s![ 0, .., ..]).fill(0.);
  ut.slice_mut(s![ 0, .., ..]).fill(0.);
  v .slice_mut(s![ 0, .., ..]).fill(0.);
  vt.slice_mut(s![ 0, .., ..]).fill(0.);
  w .slice_mut(s![ 0, .., ..]).fill(0.);
  wt.slice_mut(s![ 0, .., ..]).fill(0.);

  u .slice_mut(s![-1, .., ..]).fill(0.);
  ut.slice_mut(s![-1, .., ..]).fill(0.);
  v .slice_mut(s![-1, .., ..]).fill(0.);
  vt.slice_mut(s![-1, .., ..]).fill(0.);
  w .slice_mut(s![-1, .., ..]).fill(0.);
  wt.slice_mut(s![-1, .., ..]).fill(0.);

  u .slice_mut(s![..,  0, ..]).fill(0.);
  ut.slice_mut(s![..,  0, ..]).fill(0.);
  v .slice_mut(s![..,  0, ..]).fill(0.);
  vt.slice_mut(s![..,  0, ..]).fill(0.);
  w .slice_mut(s![..,  0, ..]).fill(0.);
  wt.slice_mut(s![..,  0, ..]).fill(0.);

  u .slice_mut(s![.., -1, ..]).fill(0.);
  ut.slice_mut(s![.., -1, ..]).fill(0.);
  v .slice_mut(s![.., -1, ..]).fill(0.);
  vt.slice_mut(s![.., -1, ..]).fill(0.);
  w .slice_mut(s![.., -1, ..]).fill(0.);
  wt.slice_mut(s![.., -1, ..]).fill(0.);
}
// 境界条件の更新（内側値をコピー）
#[allow(clippy::reversed_empty_ranges)]
fn boundary_update(p: &mut Array3::<T>) {
  let mut pv = p.slice(s![ 1,1..-1,1..-1])
                 .to_owned()
  ;
  p.slice_mut(s![ 0,1..-1,1..-1]).assign(&pv);

  pv =p.slice(s![-2,1..-1,1..-1]).to_owned();
  p.slice_mut(s![-1,1..-1,1..-1]).assign(&pv);

  pv =p.slice(s![1..-1, 1,1..-1]).to_owned();
  p.slice_mut(s![1..-1, 0,1..-1]).assign(&pv);

  pv =p.slice(s![1..-1,-2,1..-1]).to_owned();
  p.slice_mut(s![1..-1,-1,1..-1]).assign(&pv);

  pv =p.slice(s![1..-1,1..-1, 1]).to_owned();
  p.slice_mut(s![1..-1,1..-1, 0]).assign(&pv);
      
  pv =p.slice(s![1..-1,1..-1,-2]).to_owned();
  p.slice_mut(s![1..-1,1..-1,-1]).assign(&pv);
}
