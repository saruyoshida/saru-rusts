
use filter_supply_example::*;

fn main() {
// ----------------------------------------
  let mut filters = filter_supply();
  let mut cum_lhs = [0.0f32; FLC];
  // 繰返し観測
  for z in 0..2 {
    for (i, f) in filters.iter_mut()
                         .enumerate() {
      f.z_set(0, z as f32);
      f.predict();
      f.update();
      cum_lhs[i] = f.cum_lh();
   
      println!("index:{}",z);
      println!("xs:{:?}",f.x_as_slice());
      println!("Ps:{:?}",f.P_as_slice());
   
      println!("cum_lh:{}",f.cum_lh());
      println!("P[0,0]:{}",f.P(0, 0));
    }
    println!("lh:{:?}", &cum_lhs);
  }
}
// ----------------------------------------

