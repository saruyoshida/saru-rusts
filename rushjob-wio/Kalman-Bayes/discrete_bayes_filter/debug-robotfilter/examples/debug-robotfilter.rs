use core::fmt::Write;
use emb_textterm::*;
use emb_bargraph::*;
// フィルタ指定
use robot_filter::ObjObsFilter;

fn main() {
// ----------------------------------------
  // 棒グラフ設定
  let mut eb = EmbBargraph::new(
    (10, 121),           // 表示開始位置
    (300_u32, 118_u32),  // 表示サイズ
    0..10,               // X目盛レンジ
    0..1000,             // Y目盛レンジ
    (1.0, 1000.0),       // 補正率(x,y)
    (1, 100),            // 目盛刻み
    "posterior",         // タイトル
  );
  eb.mode_data();
  // 観測対象フィルタビルド
  let mut obfilter = ObjObsFilter::new();
  // 繰返し数設定
  let loop_cnt     = 4;
  // 文字表示開始回数
  let dsp_cnt      = 0;
  // 繰返し観測
  for i in 0..loop_cnt {
    // 観測対象フィルタ実行
    obfilter.iterations();
    // 途中結果表示
    if i >= dsp_cnt {
      // 文字列
      let dsp_text = ontheway_text(
        &obfilter,
        i,
      );
      println!("{}", dsp_text.as_str());

      for (i, d) in obfilter.posterior()
                              .iter()
                              .enumerate() {
         eb.set_data(i as f32, *d);
         let (x, y) = eb.data();
         println!("idx{} data{}", x, y);
      }
    }
  }
  // 最終結果文字列表示
  let dsp_text = result_text(
    &obfilter,
  );
  println!("{}", dsp_text.as_str());
  // 終了
}
// ----------------------------------------
// ----------------------------------------

// 途中結果文字列表示
fn ontheway_text(
  obfilter : &ObjObsFilter,
  i            : i32,
) -> EttString
{
  let mut dsp_text = EttString::new();
  let (index, val) = obfilter.argmax();
  dsp_text.clear();
  writeln!(
    dsp_text, 
    "Time {}: Pos {}, OPos {}, PrPos {}, Conf {}%", 
    i,
    obfilter.pos(),
    obfilter.sensor_pos(),
    index,
    val * 100.0,
  ).unwrap();

  dsp_text
}
// 最終結果文字列表示
fn result_text(
  obfilter : &ObjObsFilter,
) -> EttString
{
  let mut dsp_text = EttString::new();
  let (index, val) = obfilter.argmax();
  dsp_text.clear();
  writeln!(
    dsp_text, 
    "LastPos {}, ProbPos {}, Conf {}%", 
    obfilter.pos(),
    index,
    val * 100.0,
  ).unwrap();

  dsp_text
}