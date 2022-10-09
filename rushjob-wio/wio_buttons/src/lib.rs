#![no_std]

use wio_terminal::Button;

use heapless::consts::U16;
use heapless::String;
use core::fmt::Write;

#[derive(Default)]
pub struct SettingItem {
  cur: i32, 
  stop: i32, 
  limit: i32, 
  step: i32, 
  name: String::<U16> ,
}

impl SettingItem {
  pub fn add_current(&mut self, adder: i32) { 
    let value = self.cur + adder * self.step;
    self.set_current(value);
  }

  pub fn set_current(&mut self, value: i32) {   
    self.cur =  if value > self.limit {
      self.stop  
    } else if value < self.stop  {
      self.limit 
    } else {
      value
    };
  }
 
  pub fn get_current(&self) -> i32 {
    self.cur
  }
  
  pub fn get_name(&self) -> String::<U16> {
    self.name.clone()
  }
}

#[derive(Debug, PartialEq, Eq)]
enum ItemSelect {
  Item1,
  Item2,
  Item3,
}

#[derive(Debug, PartialEq, Eq)]
enum Adder {
  Plus, 
  Minus,
}

impl Adder {
  pub fn get_adder(&self) -> i32 {
    match *self  {
      Adder::Plus => 1,
      Adder::Minus => -1,        
    }
  }
  
  pub fn get_name(&self) -> String::<U16> {
    match *self {
      Adder::Plus => String::from("+"),
      Adder::Minus => String::from("-"),
    }
  }
}

pub struct WioButtons {
  item1: SettingItem,
  item2: SettingItem,
  item3: SettingItem,
  cur_item: ItemSelect,
  cur_button: Button,
  cur_adder:Adder,
}

impl WioButtons {
  pub fn new(
    item1: SettingItem,
    item2: SettingItem,
    item3: SettingItem,
    ) -> WioButtons {
      let buttons = WioButtons {
        item1,
        item2,
        item3,
        cur_item: ItemSelect::Item1,
        cur_button: Button::TopLeft,
        cur_adder: Adder::Plus,
    };
    buttons
  }

  pub fn build(
    tuple1: (i32, i32, i32, i32, String<U16>),
    tuple2: (i32, i32, i32, i32, String<U16>),
    tuple3: (i32, i32, i32, i32, String<U16>),
    ) -> WioButtons {
      let item1 = SettingItem {
        cur: tuple1.0,
        stop: tuple1.1,
        limit: tuple1.2,
        step: tuple1.3,
        name: tuple1.4};
      let item2 = SettingItem {
        cur: tuple2.0,
        stop: tuple2.1,
        limit: tuple2.2,
        step: tuple2.3,
        name: tuple2.4};
      let item3 = SettingItem {
        cur: tuple3.0,
        stop: tuple3.1,
        limit: tuple3.2,
        step: tuple3.3,
        name: tuple3.4};

      WioButtons::new(item1, item2, item3)
    }
           
  pub fn reset_value(&mut self, 
    val: (i32, i32, i32)) {
    self.item1.set_current(val.0);
    self.item2.set_current(val.1);
    self.item3.set_current(val.2);
  }

  pub fn get_value(&self) -> (i32, i32, i32) {
    (self.item1.get_current(),
     self.item2.get_current(),
     self.item3.get_current(),
    )
  }

  pub fn get_state(&self) -> String::<U16> {
    let item_text = 
      match self.cur_item  {
        ItemSelect::Item1 =>   
          self.item1.get_name(),
        ItemSelect::Item2 => 
          self.item2.get_name(),
        ItemSelect::Item3 => 
          self.item3.get_name(),
      };
        
    let mut textbuffer = String::<U16>::new();
    write!(
      textbuffer,
      "{}:{}",
      item_text,
      self.cur_adder.get_name()
    )
    .ok()
    .unwrap();
    textbuffer
  }

  pub fn add_current(&mut self) {
    let  s = 
      match self.cur_item {
        ItemSelect::Item1 => &mut self.item1,
        ItemSelect::Item2 => &mut self.item2,
        ItemSelect::Item3 => &mut self.item3,
      };
    s.add_current(
      self.cur_adder.get_adder()
    );
  }
}

impl WioButtons {
  pub fn button_pulled(&mut self, 
         button: Button) 
     -> Option<(i32, i32, i32)> {
     if self.cur_button == button {
       None
     } else {
       self.cur_button = button;
       match self.cur_button {
         Button::TopLeft | 
         Button::TopMiddle => {
           self.add_current();
           Some(self.get_value())},
         Button::Left => {
           self.cur_item = ItemSelect::Item1;
           None},
         Button::Click => {
           self.cur_item = ItemSelect::Item2;
           None},
         Button::Right => {
           self.cur_item = ItemSelect::Item3;
           None},
         Button::Up => {
           self.cur_adder = Adder::Plus;
           None},
         Button::Down => {
           self.cur_adder = Adder::Minus;
           None},
       }
     }
   }
}

