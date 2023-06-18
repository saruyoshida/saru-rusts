#![no_std]

// prior      : 事前分布 
// belief     : 事前確率分布、信念
// pdf        : 信念(belief)が確率分布
//              であることを強調した名前
// posterior  : 事後分布、事後確率分布
// predict    : 予測
// likelihood : 尤度（もっともらしさ）
// prob       : 確率分布
// offset     : 移動量
// lh_hallway : 計測値が廊下の各位置に
//              マッチする尤度を計算
// 事後分布 = (尤度 * 事前分布) / 正規化係数

pub struct DiscreteBayes;

impl DiscreteBayes {
  // 予測
  pub fn predict(
    &self,
    pdf    : &[f32],
    offset : i32,
    kernel : &[f32],
    prior  : &mut [f32],
  ) 
  {
    let n  = pdf.len();
    let kn = kernel.len();
    // カーネルの片側の幅
    let width = (kn as i32 - 1) / 2;
    // 事前分布の初期化
    for item in prior.iter_mut() {
      *item = 0.0;
    }
    // 畳み込み
    for i in 0..n { for k in 0..kn {
      let mut index = 
        (i as i32 + // 事前分布の設定箇所
         k as i32 - // カーネル数
         width    - // カーネルの片側の幅
         offset     // 移動量
        ) % n as i32; 

      if index < 0 {index += n as i32;}

      prior[i] += pdf[index as usize] * 
                  kernel[k];
    }}
  }
  
  // 計測値が廊下の各位置にマッチする
  // 尤度を計算
  pub fn lh_hallway(
    &self,
    hall: &[i32], 
    z: i32,
    z_prob: f32,
    likelihood : &mut [f32],
  )
  { // 尤度の初期化
    for item in likelihood.iter_mut() {
      *item = 1.0;
    }

    let scale = if z_prob == 1.0 {
                  1.0e8
                } else {
                  z_prob / (1.0 - z_prob)
                };
    
    for (i, val) in hall.iter().enumerate() 
    {
      if val == &z {
        likelihood[i] *= scale;
      }
    }
  }

  // 確率分布の更新
  pub fn update(
    &self,
    likelihood: &[f32], 
    prior     : &[f32],
    posterior : &mut [f32],
  )
  {
    for (i, val) in prior.iter().enumerate() 
    {
      posterior[i] = val * likelihood[i];
    }
    self.normalize(posterior);
  }

  // 正規化
  pub fn normalize(&self,pdf : &mut [f32])
  {
 
    let sum: f32 = pdf.iter().sum();
    for p in pdf.iter_mut() {
      *p /= sum;
    }
  }

  // 配列の最大値のインデックス
  pub fn argmax(
    &self,
    items: &[f32],
  ) -> (usize, f32)
  {
    let (index, max) = items.into_iter()
        .enumerate()
        .fold(
          (usize::MIN, f32::MIN), 
          |(i_a, a), (i_b, &b)| 
          {
            if b > a {(i_b, b)} 
            else     {(i_a, a)}
          }
        );
    (index, max)
  }
}