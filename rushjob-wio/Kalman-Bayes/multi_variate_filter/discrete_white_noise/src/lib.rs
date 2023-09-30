// discrete_white_noise
//
// ブロック分散はこれでいいとして、
// block_diagをどうするか。
// nalgebraで類似関数を見つけられなかった。
// 自作する？
//
// 自作案
//   型指定
//     型M:返却する行列次元,
//     型B:ブロック単位の次元
//   型間のルール
//     B in (U2, U3, U4) であること
//     M >= B         であること
//     M % B == 0     であること
//     M == Bの場合
//       ブロック単位をそのまま返却
//     M > B の場合
//       M / B回 対角にブロックを設定
//
//   手順
//     new
//       M, Bを受取り、MxMとBxBの行列を
//       ゼロ初期化で作成。
//     トレイト
//       型指定:M,Bを受取りMxMの行列を返却
//     実装
//       (b,_)=ブロック.shape()
//       (q,_)=Q.shape()
//       for i in 0..(q / b)
//         Q.fixed_view::<B, B>(i*b, i*b)
//         .copy_from(&ブロック)
//
//  自作関数は色々考えないといけず、
//  めんどくさいので暫くは以下の方法
//  でしのぐ。※利用者側で行う。
//  let (qd, bd) = (M::dim(), B::dim());
//  (0..qd/bd).for_each(|i| 
//    kf.Q.view_mut((i*bd, i*bd), (bd, bd))
//      .copy_from(&bn)
//  );

#![no_std]

extern crate nalgebra as na;
use na::base::dimension::*;
use na::OMatrix;
use na::DefaultAllocator;
use na::allocator::Allocator;
use micromath::F32Ext;

pub struct DiscreteWhiteNoise;

pub trait Noizeblock<B> 
where
  B: DimName,
  DefaultAllocator: Allocator<f32, B, B> 
{
  fn noise_block(dt: f32, var: f32)
    -> OMatrix::<f32, B, B>;
}

// U2 x U2
impl Noizeblock<U2> for DiscreteWhiteNoise
{
  fn noise_block(dt : f32, var: f32)
    -> OMatrix::<f32, U2, U2>
  { 
    let mut q =  OMatrix::<f32, U2, U2>::new(
      0.25*dt.powi(4), 0.5*dt.powi(3),
      0.5 *dt.powi(3),     dt.powi(2),
    );
    q *= var;
    q
  }
}
// [[.25*dt**4, .5*dt**3],
//  [ .5*dt**3,    dt**2]]

// U3 x U3
impl Noizeblock<U3> for DiscreteWhiteNoise
{
  fn noise_block(dt : f32, var: f32)
    -> OMatrix::<f32, U3, U3>
  { 
    let mut q =  OMatrix::<f32, U3, U3>::new(
      0.25*dt.powi(4), 
                   0.5*dt.powi(3),
                               0.5*dt.powi(2),
      0.5 *dt.powi(3), dt.powi(2), dt        ,
      0.5 *dt.powi(2), dt,         1.        ,
    );
    q *= var;
    q
  }
}
// [[.25*dt**4, .5*dt**3, .5*dt**2],
//  [ .5*dt**3,    dt**2,       dt],
//  [ .5*dt**2,       dt,        1]]

// U4 x U4
impl Noizeblock<U4> for DiscreteWhiteNoise
{
  fn noise_block(dt : f32, var: f32)
    -> OMatrix::<f32, U4, U4>
  { 
    let mut q =  OMatrix::<f32, U4, U4>::new(
      dt.powi(6)/36., 
              dt.powi(5)/12., 
                      dt.powi(4)/6., 
                              dt.powi(3)/6.,
      dt.powi(5)/12.,
              dt.powi(4)/4.,
                      dt.powi(3)/2., 
                              dt.powi(2)/2.,
      dt.powi(4)/6.,
              dt.powi(3)/2.,
                      dt.powi(2),
                              dt,
      dt.powi(3)/6.,
              dt.powi(2)/2.,
                      dt,        
                              1.,
    );
    q *= var;
    q
  }
}
// [[(dt**6)/36, (dt**5)/12, (dt**4)/6, 
//                               (dt**3)/6],
//  [(dt**5)/12, (dt**4)/4,  (dt**3)/2, 
//                               (dt**2)/2],
//  [(dt**4)/6,  (dt**3)/2,   dt**2,
//                                      dt],
//  [(dt**3)/6,  (dt**2)/2 ,  dt,        
//                                     1.]]

