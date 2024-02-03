// Itertools-numのLinspaceをf32限定でno_std
// のバージョン。
// ほぼ丸パクリなのでひっそりと置いています。
#![no_std]

#[derive(Clone, Debug)]
pub struct LinspaceF32 {
  start: f32,
  step: f32,
  index: usize,
  len: usize,
}

impl Iterator for LinspaceF32
{
  type Item = f32;

  #[inline]
  fn next(&mut self) -> Option<f32> {
    if self.index >= self.len {
      None
    } else {
      let i = self.index;
      self.index += 1;
      Some(self.start + self.step * i as f32)
    }
  }

  #[inline]
  fn size_hint(&self) -> (
    usize, Option<usize>
  )
  {
    let n = self.len - self.index;
    (n, Some(n))
  }
}

impl DoubleEndedIterator for LinspaceF32
{
  #[inline]
  fn next_back(&mut self) -> Option<f32> {
    if self.index >= self.len {
       None
    } else {
      self.len -= 1;
      let i = self.len;
      Some(self.start + self.step * i as f32)
    }
  }
}

impl ExactSizeIterator for LinspaceF32
where LinspaceF32: Iterator
{}

#[inline]
pub fn linspacef32(a: f32, b: f32, n: usize) 
  -> LinspaceF32
{
  let step = if n > 1 {
    let nf: f32 = n as f32;
    (b - a) / (nf - 1.0)
  } else {
    0.0
  };
  LinspaceF32 {
    start: a,
    step: step,
    index: 0,
    len: n,
  }
}


