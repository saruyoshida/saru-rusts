extern crate nalgebra as na;
use micromath::F32Ext;
//use na::OMatrix;

fn main() {
//  let h = na::Matrix2::new(4.0, 0.0,
//                            3.0, 5.0);
//  let h = na::Matrix2::new(4.0, 3.9,
//                           3.9, 4.0);

  // 特異値分解
  let h = na::Matrix2::new(4.0, 0.0,
                           0.0, 4.0);
    // testing both approaches
//  let svd1 = na::linalg::SVD::new(
//    h, true, true
//  );

  let svd = h.svd(true, true);

//  println!("{:?}", svd1);
//  println!("{:?}", svd2);

  println!("u00:{:?}", svd.u.unwrap().m11);
  println!("u01:{:?}", svd.u.unwrap().m12);
  println!("u10:{:?}", svd.u.unwrap().m21);
  println!("u11:{:?}", svd.u.unwrap().m22);

  let u = svd.u.unwrap();

  let th = u.m21.atan2(u.m11);    // 角度
  let s = svd.singular_values;   
  let r = s.x.sqrt();             // 半径
  let b = s.y.sqrt() / r;         // 短軸率
  let a = 1.0;                    // 長軸率

  println!("th:{:?}", th);
  println!("r:{:?}", r);
  println!("a:{:?}", a);
  println!("b:{:?}", b);


//  println!("{}", dsp_text.as_str())
}
// ----------------------------------------
// ----------------------------------------

