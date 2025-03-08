
use action_integral_struct::*;

fn main() {
  let mut action = create_trial_action(2.5);

  (15..28).map(|k| k as f32 * 0.1)
          .for_each(|k| {
             action.set_k(k);
             println!("{}, {}", 
               k,
               action.action_integral()
             );
           })
}

