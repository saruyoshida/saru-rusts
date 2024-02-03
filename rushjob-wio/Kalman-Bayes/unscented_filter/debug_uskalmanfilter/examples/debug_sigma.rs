extern crate nalgebra as na;
//use micromath::F32Ext;
use na::{OMatrix};
use na::base::dimension::*;
use ms_sigmapoints::*;
use us_transform::*;
use core::f32::consts::PI;
use na::DefaultAllocator;
use na::allocator::Allocator;

use discrete_white_noise::*;

  type M = U4; // 状態、プロセスモデル
  type G = U5; // シグマポイント 
  type B = U2;

fn main() {
  let dt = 1.0;
  let mut x = OMatrix::<f32, M, U1>::zeros();
  let f = &[1., 0., 0., 0.,
            dt, 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., dt, 1.0];
  
  let r  = 0.3 * 0.3;
  let q  = 0.02;

//  println!("x*x:{:?}", &x * &x.transpose());
//  println!("x.x:{:?}", &x.dot(&x));

  let mut p = OMatrix::<f32, M, M>::zeros();
  p.copy_from_slice(f);
//  println!("h*x:{:?}", &h * &x);


  let mut sigma = <MSSigmaPoints<M, G>>::new(
    0.1, // alpha
    0.2, // beta
    1.0, // kappa
  );

  println!("x:{:?}", &x.transpose());
  println!("P:{:?}", &p.transpose());
//  sigma.subtract = residual;
  println!("sigma1");
  let points = sigma.sigma_points(&x, &p);

 println!("points:{:?}", &points.transpose());
// println!("wm:{:?}", &sigma.Wm.transpose());
// println!("wc:{:?}", &sigma.Wc.transpose());

//  println!("c00:{:?}", c[(0,0)]);
//  println!("c01:{:?}", c[(0,1)]);
//  println!("c10:{:?}", c[(1,0)]);
//  println!("c11:{:?}", c[(1,1)]);

//  points.column_mut(M::dim() - 1)
//        .iter_mut()
//        .for_each(
//           |x| *x = normalize_angle(&x)
//         );

  let mut Q = OMatrix::<f32, M, M>::zeros();
  
  // ノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, q); 
  
  let (qd, bd) = (M::dim(), B::dim());
  (0..qd/bd).for_each(|i| 
    Q.view_mut((i*bd, i*bd), (bd, bd))
     .copy_from(&bn)
  );

    println!("Q:{}", &Q.transpose());

  let mut ut = <UsTransform<M, G>>::new();
  println!("ut");
  let (x, p) = ut.transform(
                 &points,
                 &sigma.Wm,
                 &sigma.Wc,
                 &Q,
               );
                 
  println!("x:{:?}", &x.transpose());
  println!("P:{:?}", &p.transpose());

  println!("sigma2");
  let points2 = sigma.sigma_points(
    &x.transpose(), &p);

 println!("points2:{:?}", &points2.transpose());
//  println!("x:{:?}", &x);
//  println!("P:{:?}", &p.transpose());

}