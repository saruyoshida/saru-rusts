#![no_std]

#[derive(Default)]
pub struct WioReferee {
    cmp1: i32, 
    cmp2: i32, 
    cmp3: i32, 
    as_update: bool, 
    upcnt: u32,
}

impl WioReferee {
    pub fn new() -> Self {
      WioReferee {
        cmp1: 0,
        cmp2: 0,
        cmp3: 0,
        as_update: false,
        upcnt: 0
      }
    }

    pub fn judgment(&mut self, 
              value: (i32, i32, i32)) { 
        if (self.cmp1,
            self.cmp2,
            self.cmp3) ==
           (value.0, value.1, value.2) {
           self.as_update = false;
        } else {
           self.cmp1 = value.0;
           self.cmp2 = value.1;
           self.cmp3 = value.2;
           self.as_update = true;
           if self.upcnt < 3 { 
              self.upcnt += 1;
           }
        }
     }
     
     pub fn as_update(&self) -> bool {
         self.as_update
     }

     pub fn as_first(&self) -> bool {
         if self.upcnt  < 2 {
            true
         } else {
           false
         }
     }
}    
