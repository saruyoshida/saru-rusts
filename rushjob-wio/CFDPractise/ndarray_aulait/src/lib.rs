// mdarray用オレオレ関数
#![no_std]
use ndarray::prelude::*;

// ----------------------------------------
// ﾏｽｷﾝｸﾞﾃｰﾌﾞﾙを使ったコピー Array2限定
pub fn maskcopya2<T>(
  to  : (&mut Array2<T>, &Array2<bool>),
  from: (&Array2<T>, &Array2<bool>),
)
where T: Copy
{
  to.0.iter_mut().zip(to.1.iter())
  .filter(|(_, mask)| **mask)
  .map(|(a, _)| a)
  .zip(
    from.0.iter().zip(from.1.iter())
    .filter(|(_, mask)| **mask)
    .map(|(b, _)| b)
  ).for_each(|(a, b)| *a = *b);
}
// ----------------------------------------
// rollのrust-ndarrayバージョン Array2限定
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla2<T>(
  array: &Array2<T>, 
  shift: isize, 
  axis: usize
) -> Array2<T> 
where T: Clone + Copy
{
  let (rows, cols) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }

      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![shift.., ..])
      );
      array.slice(
        s![(rows - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., shift..])
      );
      array.slice(
        s![.., (cols - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1"),
  }
}
// ArrayView2を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla2v<T>(
  array: ArrayView2<T>, 
  shift: isize, 
  axis: usize
) -> Array2<T>
where T: Clone + Copy
{
  let (rows, cols) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![shift.., ..])
      );
      array.slice(
        s![(rows - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., shift..])
      );
      array.slice(
        s![.., (cols - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1"),
  }
}
// &ArrayViewMut2を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla2vm<T>(
  array: &ArrayViewMut2<T>, 
  shift: isize, 
  axis: usize
) -> Array2<T>
where T: Clone + Copy
{
  let (rows, cols) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![shift.., ..])
      );
      array.slice(
        s![(rows - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array2::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., shift..])
      );
      array.slice(
        s![.., (cols - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1"),
  }
}
// ----------------------------------------
// rollのrust-ndarrayバージョン Array3限定
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla3<T>(
  array: &Array3<T>, 
  shift: isize, 
  axis: usize
) -> Array3<T> 
where T: Clone + Copy
{
  let (rows, cols, depth) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }

      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), .., ..]
      ).assign_to(
        result.slice_mut(s![shift.., .., ..])
      );
      array.slice(
        s![(rows - shift as usize).., .., ..]
      ).assign_to(
        result.slice_mut(s![..shift, .., ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![.., shift.., ..])
      );
      array.slice(
        s![.., (cols - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![.., ..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., .., ..(depth - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., .., shift..])
      );
      array.slice(
        s![.., .., (depth - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., .., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2"),
  }
}
// ArrayView3を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla3v<T>(
  array: ArrayView3<T>, 
  shift: isize, 
  axis: usize
) -> Array3<T> 
where T: Clone + Copy
{
  let (rows, cols, depth) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), .., ..]
      ).assign_to(
        result.slice_mut(s![shift.., .., ..])
      );
      array.slice(
        s![(rows - shift as usize).., .., ..]
      ).assign_to(
        result.slice_mut(s![..shift, .., ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![.., shift.., ..])
      );
      array.slice(
        s![.., (cols - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![.., ..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., .., ..(depth - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., .., shift..])
      );
      array.slice(
        s![.., .., (depth - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., .., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2"),
  }
}
// &ArrayViewMut3を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla3vm<T>(
  array: &ArrayViewMut3<T>, 
  shift: isize, 
  axis: usize
) -> Array3<T> 
where T: Clone + Copy
{
  let (rows, cols, depth) = array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize), .., ..]
      ).assign_to(
        result.slice_mut(s![shift.., .., ..])
      );
      array.slice(
        s![(rows - shift as usize).., .., ..]
      ).assign_to(
        result.slice_mut(s![..shift, .., ..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., ..(cols - shift as usize), ..]
      ).assign_to(
        result.slice_mut(s![.., shift.., ..])
      );
      array.slice(
        s![.., (cols - shift as usize).., ..]
      ).assign_to(
        result.slice_mut(s![.., ..shift, ..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array3::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![.., .., ..(depth - shift as usize)]
      ).assign_to(
        result.slice_mut(s![.., .., shift..])
      );
      array.slice(
        s![.., .., (depth - shift as usize)..]
      ).assign_to(
        result.slice_mut(s![.., .., ..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2"),
  }
}

// ----------------------------------------
// rollのrust-ndarrayバージョン Array4限定
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla4<T>(
  array: &Array4<T>, 
  shift: isize, 
  axis: usize
) -> Array4<T>
where T: Clone + Copy
{
  let (rows, cols, depth, forth) =array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }

      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize),..,..,..]
      ).assign_to(
        result.slice_mut(s![shift..,..,..,..])
      );
      array.slice(
        s![(rows - shift as usize)..,..,..,..]
      ).assign_to(
        result.slice_mut(s![..shift,..,..,..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..(cols - shift as usize),..,..]
      ).assign_to(
        result.slice_mut(s![..,shift..,..,..])
      );
      array.slice(
        s![..,(cols - shift as usize)..,..,..]
      ).assign_to(
        result.slice_mut(s![..,..shift,..,..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..(depth-shift as usize),..]
      ).assign_to(
        result.slice_mut(s![..,..,shift..,..])
      );
      array.slice(
        s![..,..,(depth-shift as usize)..,..]
      ).assign_to(
        result.slice_mut(s![..,..,..shift,..])
      );
      unsafe { result.assume_init() }
    }
    3 => { // shift 4th
      let len = forth as isize;
      if len == 0 {
        return array.clone();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.clone();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..,..(forth-shift as usize)]
      ).assign_to(
        result.slice_mut(s![..,..,..,shift..])
      );
      array.slice(
        s![..,..,..,(forth-shift as usize)..]
      ).assign_to(
        result.slice_mut(s![..,..,..,..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2 or 3"),
  }
}
// ArrayView4を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla4v<T>(
  array: ArrayView4<T>, 
  shift: isize, 
  axis: usize
) -> Array4<T>
where T: Clone + Copy
{
  let (rows, cols, depth, forth) =array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize),..,..,..]
      ).assign_to(
        result.slice_mut(s![shift..,..,..,..])
      );
      array.slice(
        s![(rows - shift as usize)..,..,..,..]
      ).assign_to(
        result.slice_mut(s![..shift,..,..,..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..(cols - shift as usize),..,..]
      ).assign_to(
        result.slice_mut(s![..,shift..,..,..])
      );
      array.slice(
        s![..,(cols - shift as usize)..,..,..]
      ).assign_to(
        result.slice_mut(s![..,..shift,..,..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..(depth-shift as usize),..]
      ).assign_to(
        result.slice_mut(s![..,..,shift..,..])
      );
      array.slice(
        s![..,..,(depth-shift as usize)..,..]
      ).assign_to(
        result.slice_mut(s![..,..,..shift,..])
      );
      unsafe { result.assume_init() }
    }
    3 => { // shift 4th
      let len = forth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..,..(forth-shift as usize)]
      ).assign_to(
        result.slice_mut(s![..,..,..,shift..])
      );
      array.slice(
        s![..,..,..,(forth-shift as usize)..]
      ).assign_to(
        result.slice_mut(s![..,..,..,..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2 or 3"),
  }
}
// &ArrayViewMut4を受け取った場合
#[allow(clippy::reversed_empty_ranges)]
pub fn rolla4vm<T>(
  array: &ArrayViewMut4<T>, 
  shift: isize, 
  axis: usize
) -> Array4<T>
where T: Clone + Copy
{
  let (rows, cols, depth, forth) =array.dim();

  match axis {
    0 => { // shift rows
      let len = rows as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }

      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..(rows - shift as usize),..,..,..]
      ).assign_to(
        result.slice_mut(s![shift..,..,..,..])
      );
      array.slice(
        s![(rows - shift as usize)..,..,..,..]
      ).assign_to(
        result.slice_mut(s![..shift,..,..,..])
      );
      unsafe { result.assume_init() }
    }
    1 => { // shift columns
      let len = cols as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..(cols - shift as usize),..,..]
      ).assign_to(
        result.slice_mut(s![..,shift..,..,..])
      );
      array.slice(
        s![..,(cols - shift as usize)..,..,..]
      ).assign_to(
        result.slice_mut(s![..,..shift,..,..])
      );
      unsafe { result.assume_init() }
    }
    2 => { // shift depth
      let len = depth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..(depth-shift as usize),..]
      ).assign_to(
        result.slice_mut(s![..,..,shift..,..])
      );
      array.slice(
        s![..,..,(depth-shift as usize)..,..]
      ).assign_to(
        result.slice_mut(s![..,..,..shift,..])
      );
      unsafe { result.assume_init() }
    }
    3 => { // shift 4th
      let len = forth as isize;
      if len == 0 {
        return array.to_owned();
      }
      let shift = ((shift % len) + len) % len;
      if shift == 0 {
        return array.to_owned();
      }
      let mut result = Array4::<T>::uninit(
        array.dim()
      );
      array.slice(
        s![..,..,..,..(forth-shift as usize)]
      ).assign_to(
        result.slice_mut(s![..,..,..,shift..])
      );
      array.slice(
        s![..,..,..,(forth-shift as usize)..]
      ).assign_to(
        result.slice_mut(s![..,..,..,..shift])
      );
      unsafe { result.assume_init() }
    }
    _ => panic!("Axis 0 or 1 or 2 or 3"),
  }
}