use core::f32::consts::PI;

use rand_distr::{Distribution, Uniform};
use rand::prelude::*;
use micromath::F32Ext;

fn main() {
  const G: usize = 4;
  const M: usize = 3;
  const D: usize = 2;

  let mut rng = SmallRng::from_seed([42 ;32]);
  let mut sum: f32 = 0.0;
  for _ in 0..20 {
    let a: f32 = rng.gen();
    println!("{:?}", a);
    sum += a;
  }
  println!("sum{:?}", sum);

  let mut pt: [[f32; M]; G] =
               [[0.1, 0.2, 0.03],
                [0.4, 0.5, 0.06],
                [0.7, 0.8, 0.09],
                [1.1, 1.2, 0.11]];

  let mut wg: [f32; G] = [1.0 / G as f32; G];
  let sensor_std_err = 0.1;
  let landmarks = [[-1., 2.], [5., 10.],
                   [12.,14.], [18.,21.]];
  let mut robot_pos = [0., 0.];


    let lmz = landmarks.into_iter().map(|lm|
                (lm, 
                 multi_norm(
                   robot_pos.into_iter(),
                   lm.into_iter(),
                   D as i32,
                 ) + rng.gen::<f32>()
                   * sensor_std_err
                )
              );

    let mut wgsum = 0.0;
    pt.into_iter().zip(lmz)
        .enumerate()
        .for_each(|(r, (pt, (lm, z)))| {
     println!("pt:{:?}, lm:{:?}, z:{:?}", 
              pt, lm, z);
           // 粒子とﾗﾝﾄﾞﾏｰｸの直距離を算,出
           let dist = multi_norm(
                        pt.into_iter()
                          .take(D),
                        lm.into_iter(),
                        D as i32,
                      );
      println!("dist:{}", dist);
           // 観測値との比を重みに掛ける
      println!("wg:{}", wg[r]);
           wg[r] = wg[r] *
                        pdf(
                          dist,
                          sensor_std_err,
                          z,
                        )          
                        + f32::MIN_POSITIVE
                        ;
      println!("wg:{}", wg[r]);
           wgsum += wg[r];
        });
      println!("wgsum:{}", wgsum);
    // 重みの正規化
    (0..wg.len()).for_each(|r| 
      wg[r] /= wgsum
    );
/*
//  let mut ptv = multinomal_resample::<M,G>(
//  let mut ptv = residual_resample::<M,G>(
//  let mut ptv = stratified_resample::<M,G>(
let mut ptv = systematic_resample::<M,G>(
      &mut wg, &pt, &mut rng);

  core::mem::swap(
      &mut pt,
      &mut ptv,
  );

  for d in pt.iter() {
    println!("{:?}", d)
  }

  let between = Uniform::from(0.0..1.0);

  for i in 0..1000 {
    let u =  between.sample(&mut rng);
    if i < 10 || i > 990 {
      println!("i:{:?}, u:{:?}", i, u);
    }  
  }
*/
}
  // 直距離算出
  pub fn multi_norm(
    a: impl Iterator<Item=f32>,
    b: impl Iterator<Item=f32>,
    d: i32,
  ) -> f32
  {
    let df = 1.0 / d as f32;  // 冪乗根用

    a.zip(b)
     .map(|(a, b)| (a - b).abs().powi(d))
     .fold(0.0, |x, y| x + y)
     .powf(df)
  }
  // 確率密度関数:正規分布
  pub fn pdf(
    mean: f32,
    var : f32,
    x   : f32,
  ) -> f32
  {
    let y = (x - mean) / var;
    // 1/√2πσ*exp(-(x-μ)^2/2σ^2)
    (1.0 / 
      (2.0 * PI * var).sqrt() *
      (-1.0 * (y - mean).powi(2) / 
              (2.0 * var.powi(2))
      ).exp()
    ) / var
  }
/*
pub fn multinomal_resample
  <const M : usize,
   const G : usize>
(
  wg : &mut [f32; G],
pt : &[[f32; M]; G],
  rng: &mut SmallRng,
) -> [[f32; M]; G]
{
  // cumsum
  (1..wg.len()).for_each(|i| 
    wg[i] = wg[i] + wg[i-1]
  );
  wg[G-1] = 1.0;

  let mut v = [[0.0_f32; M]; G];
  (0..wg.len()).for_each(|i| v[i] = 
    pt[wg.partition_point(|&x| x < rng.gen())]
  );
  v
}

pub fn residual_resample
  <const M : usize,
   const G : usize>
(
  wg : &mut [f32; G],
  pt : &[[f32; M]; G],
  rng: &mut SmallRng,
) -> [[f32; M]; G]
{
  let g = G as f32;
  let mut v = [[0.0; M]; G];
  let mut k = 0;

  for i in 0..wg.len() {
    for _ in 0..(wg[i] * g) as usize {
       v[k] = pt[k];
       k += 1;
    }
  }

  let mut sum = 0.0;
  (0..wg.len()).for_each(|i| {
    wg[i] = (wg[i] * g).trunc();
    sum += wg[i]
  });
  // 正規化とcumsumをいっぺんに行う
  wg[0] = wg[0] / sum;
  (1..wg.len()).for_each(|i| 
     wg[i] = wg[i] / sum + wg[i-1]
  );
  wg[G-1] = 1.0;

  (k..wg.len()).for_each(|i| v[i] = 
    pt[wg.partition_point(|&x| x < rng.gen())]
  );
  v
}
pub fn stratified_resample
  <const M : usize,
   const G : usize>
(
  wg : &mut [f32; G],
  pt : &[[f32; M]; G],
  rng: &mut SmallRng,
) -> [[f32; M]; G]
{
  let g = G as f32;
  let pos = (0..G).map(|r|
    (r,  (r as f32 + rng.gen::<f32>()) / g)
  );
  // cumsum
  (1..wg.len()).for_each(|i| 
    wg[i] = wg[i] + wg[i-1]
  );
  
let mut v = [[0.0; M]; G];
  let mut j = 0;
  pos.for_each(|(i, p)| {
    while !(p < wg[j]) {j += 1}
    v[i] = pt[j];
  });
  v
}
pub fn systematic_resample
  <const M : usize,
   const G : usize>
(
  wg : &mut [f32; G],
  pt : &[[f32; M]; G],
  rng: &mut SmallRng,
) -> [[f32; M]; G]
{
  let g = G as f32;
  let d = rng.gen::<f32>();
  let pos = (0..G).map(|r|
    (r,  (r as f32 + d) / g)
  );
  // cumsum
  (1..wg.len()).for_each(|i| 
    wg[i] = wg[i] + wg[i-1]
  );
  
let mut v = [[0.0; M]; G];
  let mut j = 0;
  pos.for_each(|(i, p)| {
    while !(p < wg[j]) {j += 1}
    v[i] = pt[j];
  });
  v
}
*/
/*
def stratified_resample(weights):
    N = len(weights)
    # N 個の小区間を作り、それぞれの内部にランダムな点を生成する。
    positions = (random(N) + range(N)) / N

    indexes = np.zeros(N, 'i')
    cumulative_sum = np.cumsum(weights)
    i, j = 0, 0
    while i < N:
        if positions[i] < cumulative_sum[j]:
            indexes[i] = j
            i += 1
        else:
            j += 1
    return indexes

def residual_resample(weights):
    N = len(weights)
    indexes = np.zeros(N, 'i')

    # 粒子 i を int(N*w)[i] 個だけ選択する。
    Nw = N*weights
    num_copies = Nw.astype(int)
    k = 0
    for i in range(N):
        for _ in range(num_copies[i]): # num_copies[i] 個だけ i を選択する。
            indexes[k] = i
            k += 1

    # 残差を重みとした多項再サンプリングで残りの部分を埋める。
    residual = Nw - num_copies # 小数部分を求める。
    residual /= sum(residual)  # 正規化する。
    cumulative_sum = np.cumsum(residual)
    cumulative_sum[-1] = 1.    # 和が 1 であることを保証する。
    indexes[k:N] = np.searchsorted(cumulative_sum, random(N-k))

    return indexes
*/
