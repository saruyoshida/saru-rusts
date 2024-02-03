use itertools_num::linspace;

fn main() {
  println!("x0.0,y1.0:{}",1.0_f32.atan2(0.0));
  println!("x0.5,y0.5:{}",0.5_f32.atan2(0.5));
  println!("x1.0,y0.0:{}",
           0.0_f32.atan2(1.0));
  println!("x0.5,y-0.5:{}",
           -0.5_f32.atan2(0.5));
  println!("x0.0,y-1.0:{}",
           -1.0_f32.atan2(0.0));
  println!("x-0.5,y-0.5:{}",
           -0.5_f32.atan2(-0.5));
  println!("x-1.0,y0.0:{}",
           0.0_f32.atan2(-1.0));
  println!("x-0.5,y0.5:{}",
           0.5_f32.atan2(-0.5));
/*
  let mut cmd1 = linspace::<f32>(
                   0.001, 1.1, 30
                 );
*/
  let cmd1 = Tests::new(make_cmd());
  cmd1.it.take(5).for_each(|v|
      println!("V:{:?}", v)
  );
}
pub struct Tests<T> {
  pub it: T
}
impl<T> Tests<T>
  where T: Iterator<Item=[f32; 2]>
{
  pub fn new(it: T) -> Self {
    Self{it}
  }
}

fn make_cmd() -> impl Iterator<Item=[f32; 2]>
{
  let cmd1 = linspace::<f32>(
               -2.,0., 15
             ).map(|c| [c, 0.0]);
  cmd1
}