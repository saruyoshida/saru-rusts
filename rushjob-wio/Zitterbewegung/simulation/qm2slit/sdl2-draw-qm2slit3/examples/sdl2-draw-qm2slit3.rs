pub use embedded_graphics::{
  pixelcolor::Rgb565,
  primitives::{Rectangle,PrimitiveStyle},
  prelude::*,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, 
    SimulatorDisplay, 
    SimulatorEvent, 
    Window,
};
pub use std::{thread, time::Duration};

// 電子の自由運動
use qms::*;
// グラフ供給
use graph_supply_qm::*;

type T = f32;
// ----------------------------------------
fn main() -> 
  Result<(), core::convert::Infallible>
{
  let mut display: SimulatorDisplay<Rgb565> 
    = SimulatorDisplay::new(
                          Size::new(320, 240)
                        );
  let output_settings = OutputSettingsBuilder
                        ::new().scale(2)
                               .build();
  let mut window = Window::new(
                     "PrticleFilter", 
                     &output_settings
                   );
  // 画面クリア ---------------------------
  Rectangle::new(
    Point::new(0,0), Size::new(320, 240)
  ).into_styled(
    PrimitiveStyle::with_fill(Rgb565::BLACK)
  ).draw(&mut display).unwrap();
  // グラフ供給 ---------------------------
  let (mut graph_box, mut graph_obj) =
    graph_supply(
       0..300,      // x目盛
       -125..75,    // y目盛
       (100., 10.), // 補正率
       (100, 25),   // 目盛刻み
    );
  let gb = &mut graph_box[0];
  let g =  &mut graph_obj[0][0];
  // 目盛表示
  gb.mode_scale();
  gb.draw(&mut display).unwrap();
  gb.mode_clear();
  // グラフ立体視化(補正率x,   y,   z)
  let p3 = Plot3D::new((100., 10., 40.));
  p3.draw(&mut display).unwrap();
  window.update(&display);
  // 結果描画 -----------------------------
  let mut seed: u8 = 0; // シード値設定
  // スリット設定
  let a: T = 0.3;       // スリット巾
  let s: T = 0.3;       // ｽﾘｯﾄ間隔半値
  let s2   = s.powi(2); // s^2
  let s4   = s.powi(4); // s^4
  let aps  = a+s;       // a+s
  
  // 二重スリット(Excelの内容)
  let d = move |yt:T, t: T|
    // t-2s^2 / t^2+4s^4 Y(t)
    (t-2.*s2) / (t*t+4.*s4) * yt +
    ((  // sinh(4s^2sY(t) / 4s^4+t^2) *
        //     (t-2s^2    / t^2+4s^4)
      ((4.*s2*s*yt) / (4.*s4+t*t)).sinh() *
      ((t-2.*s2)    / (t*t+4.*s4))
      - // sin(2tsY(t) / 4s^4+t^2) *
        //    (t+2s^2  / t^2+4s^4)
      ((2.*t*s*yt)  / (4.*s4+t*t)).sin()  *
      ((t+2.*s2)    / (t*t+4.*s4))
     ) /
     (  // cosh(4s^2sY(t) / 4s^4+t^2) +
        //  cos(2tsY(t)   / 4s^4+t^2)
      ((4.*s2*s*yt) / (4.*s4+t*t)).cosh() +
      ((2.*t*s*yt)  / (4.*s4+t*t)).cos()
     )
    ) * s
  ;
  // 見本経路40本描画
  (0..40)
  .map(|z| z as T)
  .map(|z| (z, -aps+(1./20.)*aps*z))// y初期値
  .for_each(|(z, y0)| {
    g.reset_data();
    seed = (seed+43) % 255; // ｼｰﾄﾞ値変更
    // 電子の自由運動
    qms(d, y0, 0.01, 300, seed)
    // 見本経路描画
    .for_each(|(x, y)| {
       let (x, y) = p3.conv(
         x, y, (40.-z*1.5)/40.
       );
       g.set_data(x, y); 
       graph_supply_draw!(g, display, Go1,);
    });
    window.update(&display);
  });
  // -------------------------------------
  'running: loop {
    if window.events()
       .any(|e| e == SimulatorEvent::Quit) 
    {
      break 'running Ok(());
    }
  }
}
