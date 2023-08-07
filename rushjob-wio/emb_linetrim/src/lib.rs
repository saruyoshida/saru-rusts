#![no_std]

use embedded_graphics::geometry::Point;
use embedded_graphics::primitives::Rectangle;
use heapless::Vec;

#[derive(Debug, PartialEq, Clone)]
struct Pof2{pub x : f32, pub y : f32}
impl Pof2 {
  pub fn new(x : f32, y : f32) -> Self {
    Pof2{x, y}
  }
}
#[derive(Clone)]
pub struct EmbLineTrim {
  area      : Rectangle,
  dspbox    : [Pof2; 5],
}

impl EmbLineTrim {
  pub fn new (
    area : Rectangle,
  ) -> Self 
  { 
    let top_left_x = area.top_left.x as f32;
    let top_left_y = area.top_left.y as f32;
    let size_width = area.size.width as f32;
    let size_height= area.size.height as f32;
    EmbLineTrim {
      area,
      dspbox:
        [Pof2::new(top_left_x,  top_left_y),
         Pof2::new(top_left_x + size_width,
                                top_left_y),
         Pof2::new(top_left_x + size_width,
                   top_left_y + size_height),
         Pof2::new(top_left_x,
                   top_left_y + size_height),
         Pof2::new(top_left_x,  top_left_y),
        ],
    }
  }
  // ラインを表示枠でトリム
  pub fn line_trim(
    &self,
    ps : &mut Point,
    pe : &mut Point,
  ) -> bool
  {
    // 描画エリア範囲内チェック
    let iscin = self.area.contains(*pe);
    let ispin = self.area.contains(*ps);

    // 今回範囲内、前回範囲内なら早期リターン
    if iscin && ispin {return true;}
    // 線分と四角形の交点を取得
    let mut cp = self.cross(pe, ps);
    // 交点が見つからない場合早期リターン
    if cp.len() == 0 {
       if !iscin && !ispin {
         return false;
       } else {
         return true;
       }
    }
    // 交点が1つの場合
    if cp.len() == 1 || 
       (cp.len() == 2 && 
        cp[0] == cp[1]
       )
    {
      if !ispin {
        // 前回範囲外の場合
        *ps = cp.pop().unwrap();
      } else {
        // 今回範囲外の場合
        *pe = cp.pop().unwrap();
      }
    } else {
    // 交点が2つの場合、x値が小さい方を始点
      if cp[0].x < cp[1].x {
        *ps = cp.pop().unwrap();
        *pe = cp.pop().unwrap();
      } else{
        *pe = cp.pop().unwrap();
        *ps = cp.pop().unwrap();
      }
    }
    true
  }
  // 四角形と線分の交点を求める
  fn cross(
    &self,
    p     : &Point,
    pp    : &Point,
  ) -> Vec<Point, 2>
  {
    let mut cross_point = 
      Vec::<Point, 2>::new();

    #[allow(non_snake_case)]
    let L1 = Pof2::new(
      pp.x as f32, pp.y as f32
    );

    #[allow(non_snake_case)]
    let L2 = Pof2::new(
      p.x as f32, p.y as f32
    );

    for i in 0..(self.dspbox.len() - 1) {
      #[allow(non_snake_case)]
      let L3 = &self.dspbox[i];

      #[allow(non_snake_case)]
      let L4 = &self.dspbox[i + 1];

      let ksi = 
        (L4.y - L3.y) * (L4.x - L1.x) -
        (L4.x - L3.x) * (L4.y - L1.y);

      let eta =
        (L2.x - L1.x) * (L4.y - L1.y) -
        (L2.y - L1.y) * (L4.x - L1.x);

      let delta = 
        (L2.x - L1.x) * (L4.y - L3.y) -
        (L2.y - L1.y) * (L4.x - L3.x);

      let ramda = ksi / delta;
      let mu    = eta / delta;

      if ramda >= 0.0 && ramda <= 1.0 && 
         mu    >= 0.0 && mu    <= 1.0
      {
        cross_point.push(
          Point::new(
            (L1.x + ramda * (L2.x - L1.x)) 
            as i32,
            (L1.y + ramda * (L2.y - L1.y)) 
            as i32,
          )
        ).unwrap();
      }
    }
    cross_point
  }
}


