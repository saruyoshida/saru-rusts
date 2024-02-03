
use core::f32::consts::PI;
use micromath::F32Ext;

use us_kalmanfilter::*;
use discrete_white_noise::*;
use radaraccsim::*;

// ========================================
// カルマンフィルタ次元設定
type M = U3; // 状態、プロセスモデル
type N = U2; // 観測値
type C = U1; // 制御入力
type B = U2; // プロセスノイズブロック
type G = U7; // シグマ点数 Mx2+1
type LR= U1; // 未使用
type LC= U1; // 未使用
// シミュレータ設定
const RADAR_POS:[f32; 2] = [0., 0.];
// ========================================
fn main() {
// ========================================
// シミュレーション設定
  let dt = 3.0;
  let range_std = 5.;
  let elevation_angle_std = 0.5/180. * PI;
// シミュレータ
  // レーダー
  let mut radar = RadarStation::new();
  radar.pos = RADAR_POS;
  radar.range_std = range_std;
  radar.elev_angle_std = elevation_angle_std;
  // 航空機
  let mut ac = ACSim::new();
  ac.pos = [0.  , 1000.];
  ac.vel = [100.,    0.];
  ac.vel_std = 0.02;
  ac.dt  = dt;
// ========================================
// 無香料カルマンフィルタ設定
  // シグマポイント
  let sg = <MSSigmaPoints<M, G>>::new(
    0.1, // alpha
    2.0, // beta
    0.0, // kappa
  );
  // 無香料変換(状態)
  let utx = <UsTransform<M, G>>::new();
  // 無香料変換(観測)
  let utz = <UsTransform<N, G>>::new();
  // 無香料フィルタ
  let mut ukf = 
    <UsKalmanFilter<M, N, C, G, LR, LC>>
    ::new(sg, utx, utz);
  // 状態関数
  ukf.F.copy_from_slice(
    &[1.0, 0.0, 0.0, 
      dt , 1.0, 0.0,
      0.0, 0.0, 1.0]
  );
  // 観測関数
  ukf.hx = h_radar;
  // 観測値ノイズ
  ukf.R.set_partial_diagonal(
    [range_std.powi(2),
     elevation_angle_std.powi(2)]
    .into_iter()
  );
  // 状態変数
  ukf.x.copy_from_slice(&[0., 90., 1100.]);
  // 状態共分散
  ukf.P.set_partial_diagonal(
    [300.0.powi(2), 
     30.0.powi(2), 
     150.0.powi(2)
    ].into_iter()
  );
  // プロセスノイズ設定
    // ノイズブロック作成
  let bn: OMatrix<f32, B, B> =  
    DiscreteWhiteNoise::noise_block(dt, 0.1); 
    // プロセスノイズ設定
  let bd = B::dim();
  ukf.Q.view_mut((0, 0), (bd, bd))
       .copy_from(&bn);
  ukf.Q[(bd, bd)] = 0.1;
// ========================================
  // 繰返し観測
  for i in (0..360 + dt as usize)
           .step_by(dt as usize) {
    ac.update();
    ukf.z.copy_from_slice(
     &(radar.noisy_reading(ac.pos))
    );
    ukf.predict();
    ukf.update();

    println!("time:{},x:{:?}", 
             i,&ukf.x.transpose());
  }
  // 終了
}
// ========================================
// 無香料カルマンフィルタ関数定義
// 観測関数
#[allow(non_snake_case)]
fn h_radar<M, N, LR, LC>(
  x:  &OMatrix<f32, M, U1>,
  _H: &OMatrix<f32, N, M>,
  _zt:&OMatrix<f32, LR, LC>,
) -> OMatrix<f32, N, U1>
where
  M : DimName,
  N : DimName,
  LR: DimName,
  LC: DimName,
  DefaultAllocator: 
    Allocator<f32, M, U1> +
    Allocator<f32, N, M>  +
    Allocator<f32, N, U1> +
    Allocator<f32, LR, LC> 
{
  // 予測値の位置と高度を、観測値の直距離と
  // 仰角に変換する

  // レーダーからの距離
  let dx = x[(0, 0)] - RADAR_POS[0]; // 位置
  let dy = x[(2, 0)] - RADAR_POS[1]; // 高度
  // 直距離
  let slant_range = (dx.powi(2) + dy.powi(2))
                    .sqrt();
  // 仰角
  let elevation_angle = dy.atan2(dx);
  // 予測値を観測値形式に変換した行列を返却
  let mut h = OMatrix::<f32, N, U1>::zeros();
  h.copy_from_slice(
    &[slant_range, elevation_angle]
  );
  h
}

