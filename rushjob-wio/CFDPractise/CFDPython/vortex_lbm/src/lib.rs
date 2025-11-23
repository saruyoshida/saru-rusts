// ndarray自主練②
#![no_std]

#[allow(unused_imports)]
use num_traits::Float;
use ndarray::{s, Array2};
use ndarray_aulait::*;

type T = f32;
// 格子ボルツマン法
#[allow(clippy::reversed_empty_ranges)]
#[allow(non_snake_case)]
pub fn vortex_lbm(
  height   : usize,  // 格子行
  width    : usize,  // 格子列
  viscosity: T,      // 液体粘度
  u0       : T,      // 初速と流入速度 
)-> impl Iterator<Item=Array2<T>>
{
  // 緩和パラメータ
  let omega = 1. / (3.*viscosity + 0.5);
  // 格子ボルツマン加重係数
  let four9ths = 4./9.;
  let one9th   = 1./9.;
  let one36th  = 1./36.;
  // u0の二乗
  let u0p2 = u0*u0;
  // すべての配列を右肩上がりの流れに初期化
  // 9方向の粒子密度
  let arr = Array2::<T>::ones((height,width));
  let mut n0 = four9ths*(&arr - 1.5*u0p2);
  let mut nN = one9th * (&arr - 1.5*u0p2);
  let mut nS = one9th * (&arr - 1.5*u0p2);
  let mut nE = one9th * (
    &arr + 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  let mut nW = one9th * (
    &arr - 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  let mut nNE = one36th * (
    &arr + 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  let mut nSE = one36th * (
    &arr + 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  let mut nNW = one36th * (
    &arr - 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  let mut nSW = one36th * (
    &arr - 3.*u0 + 4.5*u0p2 - 1.5*u0p2
  );
  // 障壁を初期化
  // 障壁箇所はtrue
  let mut barrier =         
    Array2::<bool>::default((height,width))
  ;
  barrier.slice_mut(
    s![(height/2)-8..(height/2)+8, height/2]
  ).fill(true);

  let barrierN  = rolla2(&barrier,   1, 0);
  let barrierS  = rolla2(&barrier,  -1, 0);
  let barrierE  = rolla2(&barrier,   1, 1);
  let barrierW  = rolla2(&barrier,  -1, 1);
  let barrierNE = rolla2(&barrierN,  1, 1);
  let barrierNW = rolla2(&barrierN, -1, 1);
  let barrierSE = rolla2(&barrierS,  1, 1);
  let barrierSW = rolla2(&barrierS, -1, 1);
  // すべてのパーティクルを進行方向に沿って
  // 一歩ずつ移動させる(pbc)：
  // 無限イテレータ
  (0..).scan(0 , move |_, _| {
    // stream --------------------------
    // 軸0は南北、＋方向は北
    nN  = rolla2(&nN,   1, 0);
    nNE = rolla2(&nNE,  1, 0);
    nNW = rolla2(&nNW,  1, 0);
    nS  = rolla2(&nS,  -1, 0);
    nSE = rolla2(&nSE, -1, 0);
    nSW = rolla2(&nSW, -1, 0);
    // 軸1は東西、＋方向は東
    nE  = rolla2(&nE,   1, 1);
    nNE = rolla2(&nNE,  1, 1);
    nSE = rolla2(&nSE,  1, 1);
    nW  = rolla2(&nW,  -1, 1);
    nNW = rolla2(&nNW, -1, 1);
    nSW = rolla2(&nSW, -1, 1);
    // バリア衝突（バウンスバック）を処理する
    // ために、トリッキーなブーリアン配列を
    // 使用する
    maskcopya2(
      (&mut nN, &barrierN), (&nS, &barrier)
    );
    maskcopya2(
      (&mut nS, &barrierS) ,(&nN, &barrier)
    );
    maskcopya2(
      (&mut nE, &barrierE), (&nW, &barrier)
    );
    maskcopya2(
      (&mut nW, &barrierW), (&nE, &barrier)
    );
    maskcopya2(
      (&mut nNE, &barrierNE), (&nSW, &barrier)
    );
    maskcopya2(
      (&mut nNW, &barrierNW), (&nSE, &barrier)
    );
    maskcopya2(
      (&mut nSE, &barrierSE), (&nNW, &barrier)
    );
    maskcopya2(
      (&mut nSW, &barrierSW), (&nNE, &barrier)
    );
    // collide ------------------------
    // 各セル内でパーティクルを衝突させて
    //速度を再分配
    let rho = &n0 + 
      &nN  + &nS  + &nE  + &nW +
      &nNE + &nSE + &nNW + &nSW
    ;
    let ux = (
      &nE + &nNE + &nSE - &nW - &nNW - &nSW
    ) / &rho
    ;
    let uy = (
      &nN + &nNE + &nNW - &nS - &nSE - &nSW
    ) / &rho
    ;
    let ux2 = &ux * &ux;
    let uy2 = &uy * &uy;
    let u2 = &ux2 + &uy2;
    let omu215 = 1. - 1.5*&u2;
    let uxuy = &ux * &uy;
    n0 = (1.-omega)*&n0 + omega * four9ths * 
         &rho * &omu215;
    nN = (1.-omega)*&nN + omega * one9th * 
         &rho * (&omu215 + 3.*&uy + 4.5*&uy2);
    nS = (1.-omega)*&nS + omega * one9th * 
         &rho * (&omu215 - 3.*&uy + 4.5*&uy2);
    nE = (1.-omega)*&nE + omega * one9th *
         &rho * (&omu215 + 3.*&ux + 4.5*&ux2);
    nW = (1.-omega)*&nW + omega * one9th * 
         &rho * (&omu215 - 3.*&ux + 4.5*&ux2);
    nNE = (1.-omega)*&nNE + omega * one36th * 
         &rho * (&omu215 + 3.*(&ux+&uy) + 
                 4.5*(&u2+2.*&uxuy))
    ;
    nNW = (1.-omega)*&nNW + omega * one36th * 
          &rho * (&omu215 + 3.*(-&ux+&uy) + 
                 4.5*(&u2-2.*&uxuy))
    ;
    nSE = (1.-omega)*&nSE + omega * one36th * 
          &rho * (&omu215 + 3.*(&ux-&uy) + 
                 4.5*(&u2-2.*&uxuy))
    ;
    nSW = (1.-omega)*&nSW + omega * one36th * 
          &rho * (&omu215 + 3.*(-&ux-&uy) + 
                 4.5*(&u2+2.*&uxuy))
    ;
    // 両端で右向きの安定した流れを強制する
    // （0、N、S成分を設定する必要はない）
    nE.slice_mut(s![..,0]).fill(
      one9th*(1.+3.*u0+4.5*u0p2-1.5*u0p2)
    );
    nW.slice_mut(s![..,0]).fill(
      one9th*(1.-3.*u0+4.5*u0p2-1.5*u0p2)
    );
    nNE.slice_mut(s![..,0]).fill(
      one36th*(1.+3.*u0+4.5*u0p2-1.5*u0p2)
    );
    nSE.slice_mut(s![..,0]).fill(
      one36th*(1.+3.*u0+4.5*u0p2-1.5*u0p2)
    );
    nNW.slice_mut(s![..,0]).fill(
      one36th*(1.-3.*u0+4.5*u0p2-1.5*u0p2)
    );
    nSW.slice_mut(s![..,0]).fill(
      one36th*(1.-3.*u0+4.5*u0p2-1.5*u0p2)
    );
    // 巨視的な速度場のカールを計算する：
    // curl
    Some(
      rolla2(&uy,-1,1) - rolla2(&uy,1,1) - 
      rolla2(&ux,-1,0) + rolla2(&ux,1,0)
    )
  })
}





