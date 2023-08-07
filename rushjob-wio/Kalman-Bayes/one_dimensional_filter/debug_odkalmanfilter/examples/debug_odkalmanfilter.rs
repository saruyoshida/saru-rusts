use core::fmt::Write;
use micromath::F32Ext;
use emb_textterm::EttString;
use one_dimensional::*;
use dogsimulation::DogSimulation;

// フィルタ指定
use od_kalmanfilter::ObjObsFilter;

fn main() {
// ----------------------------------------
// ----------------------------------------
// フィルタ関数
//  // 1.ガウス分布の積:ベイズ的アプローチ
//  let filter_fn = OneDimKalman; 
  // 2.残差カルマンゲイン形式
  let filter_fn = OneDimKalmanRest;
  // 設定値
  //   ドッグシミュレーション
  let process_var = 2.0;   // 犬の動きの分散
  let sensor_var  = 4.5;   // センサーの分散
  let x = (0.0, 400.0);    // 犬の位置
  let target_x = x.0;     
  let velocity    = 1.0;   // 単位移動量
  let dt          = 1.0;   // タイムステップ
  //   ボルトシミュレーション
//  let process_var = 0.05.powf(2.0);
//  let sensor_var  = 0.13.powf(2.0);
// let x           = (25.0, 1000.0);
//  let target_x    = 16.3;   
//  let velocity    = 0.0;  
//  let dt          = 0.0;
  // シミュレータ
  let mut target = DogSimulation::new();
  target.set_random_seed(134)
        .set_process_var(process_var)                     
        .set_measurement_var(sensor_var)
        .set_x(target_x)
        .set_velocity(velocity);
  // フィルタ
  let mut obfilter = ObjObsFilter::new(
    target,
    filter_fn
  );
  obfilter.set_process_model(
             (velocity * dt, process_var)
           )
          .set_sensor_var(sensor_var)
          .set_x(x)
          .set_dt(dt);
  // 繰返し観測
  for _ in 0..10 {
    // 観測対象フィルタ実行
    obfilter.iterations();
    // 途中文字列表示
    let dsp_text = ontheway_text(
      &obfilter,
    );
    println!("{}", dsp_text.as_str())
  }
}
// ----------------------------------------
// ----------------------------------------

// 途中結果文字列表示
fn ontheway_text<T, F>(
  obfilter : &ObjObsFilter<T, F>,
) -> EttString
  where F : OneDimFilter,
        T : OneDimSimulation,
{
  let mut dsp_text = EttString::new();
  let z = obfilter.zs(); // ノイズ有観測値
                         // 事前予測値,分散
  let (p_mean, p_var) = obfilter.prior();
                         // 事後予測値
  let (x_mean, x_var) = obfilter.xs();
  let k = obfilter.k();
  let t = obfilter.target_x();
  dsp_text.clear();
  writeln!(
    dsp_text, 
    "{:03.03}:{:03.03}:{:03.03}:{:03.03}: {:03.03}:{:03.03}:{:03.03}", 
    p_mean,
    p_var,
    z,
    x_mean,
    x_var,
    k,
    t,
  ).unwrap();

  dsp_text
}
