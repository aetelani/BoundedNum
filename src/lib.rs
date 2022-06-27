#![feature(test)]
#![feature(adt_const_params)]
extern crate test;

use std::borrow::Borrow;
use std::collections::Bound;
use std::env::Args;
use std::ops::{Range, RangeBounds, RangeInclusive, RangeToInclusive};
use std::ops::Bound::{Excluded, Included};
use crate::BoundedIntErr::{BoundError, Invalid};

/// Valid range Not Inclusive meaning that MAX value is invalid
/// eg. (0..4) is actually valid for values 0,1,2,3 and 4 is not valid.
pub struct BoundedInt<
    const LOWER: Bound<&'static isize> = { Included(&isize::MIN) },
    const UPPER: Bound<&'static isize> = { Excluded(&isize::MAX) }> (isize);

pub enum BoundedIntErr {
    BoundError,
    Invalid,
}

impl<const LOWER: Bound<&'static isize>, const UPPER: Bound<&'static isize>> RangeBounds<isize> for BoundedInt<LOWER, UPPER> {
    fn start_bound(&self) -> Bound<&isize> {
        LOWER
    }
    fn end_bound(&self) -> Bound<&isize> {
        UPPER
    }
}

impl<const LOWER: Bound<&'static isize>, const UPPER: Bound<&'static isize>> BoundedIntTrait for BoundedInt<LOWER, UPPER> {
    // Boilerplate to Self::RangeBounds::contains
    fn is_valid(&self) -> bool {
        self.contains(&self.0)
    }
    fn get_bounds() -> Range<isize> {
        let (start, end) =
            if let (Included(start_ref), Excluded(end_ref)) = (LOWER, UPPER) {
                (*start_ref, *end_ref)
            } else {
                todo!()
            };
        Range { start, end }
    }

    fn try_get(&self) -> Result<isize, BoundedIntErr> {
        if self.is_valid() {
            Ok(self.0)
        } else {
            Err(Invalid)
        }
    }

    fn try_set(&mut self, new_value: isize) -> Result<isize, BoundedIntErr> {
        if Self::get_bounds().contains(&new_value) {
            self.0 = new_value;
            Ok(self.0)
        } else {
            Err(BoundError)
        }
    }
    fn try_set_fn(&mut self, set_with_fn: &impl Fn(isize) -> isize) -> Result<isize, BoundedIntErr>
    {
        let new_value = set_with_fn(self.0);
        if self.contains(&new_value) {
            self.0 = new_value;
            Ok(self.0)
        } else {
            // Value not updated
            Err(BoundError)
        }
    }
    fn invalidate(&mut self) {
        // is_valid() is now false
        if let Excluded(v) = UPPER {
            self.0 = *v;
        } else {
            todo!()
        }
    }
}

impl UncheckedIntTrait for BoundedInt {
    unsafe fn set(&mut self, new_value: isize) -> isize {
        self.0 = new_value;
        self.0
    }
    unsafe fn set_fn(&mut self, set_with_fn: &impl Fn(isize) -> isize) -> isize {
        self.0 = set_with_fn(self.0);
        self.0
    }

    unsafe fn get(&self) -> isize {
        self.0
    }
}

pub trait UncheckedIntTrait {
    unsafe fn set(&mut self, new_value: isize) -> isize;
    unsafe fn set_fn(&mut self, set_with_fn: &impl Fn(isize) -> isize) -> isize;
    unsafe fn get(&self) -> isize;
}

pub trait BoundedIntTrait {
    fn is_valid(&self) -> bool;
    fn get_bounds() -> Range<isize>;
    fn try_get(&self) -> Result<isize, BoundedIntErr>;
    fn try_set(&mut self, new_value: isize) -> Result<isize, BoundedIntErr>;
    fn try_set_fn(&mut self, set_with_fn: &impl Fn(isize) -> isize) -> Result<isize, BoundedIntErr>;
    fn invalidate(&mut self);
}

#[cfg(test)]
mod tests {
    use std::collections::Bound::Included;
    use std::mem::size_of;
    use std::ops::Add;
    use std::ops::Bound::Excluded;
    use test::{Bencher, black_box};
    use crate::{BoundedInt, BoundedIntTrait, UncheckedIntTrait};

    #[test]
    fn it_jiggles() {
        type T = BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&5isize) }>;
        let t = BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&5isize) }>(5);
        assert!(!t.is_valid());
        let t = BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&10isize) }>(5);
        assert!(t.is_valid());
        assert_eq!(size_of::<BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&isize::MAX) }>>(), size_of::<isize>());
    }

    #[bench]
    fn it_folds_unchecked_set_fn(b: &mut Bencher) {
        let mut t = BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&isize::MAX) }>(0);
        //let mut t = BoundedInt(0);
        b.iter(|| unsafe {
            for i in -2000_isize..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn(&|v| v + i);
            }
        });
    }

    #[bench]
    fn it_folds_unchecked_set_fn_bb(b: &mut Bencher) {
        let mut t = BoundedInt(isize::MIN);
        //let mut t = BoundedInt(0);
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn(&|v| black_box(v + i));
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn_bb(b: &mut Bencher) {
        /*let mut t = BoundedInt::<{ isize::MIN }>(isize::MIN);
        b.iter(|| {
            for i in -2000..2000 {
                // 1.8 ns/iter with blackbox
                let g = t.try_set_fn(&|v| black_box(v + i));
                if let Some(t) = g.ok() {};
            }
        });*/
    }

    #[bench]
    fn it_folds_try_set_fn(b: &mut Bencher) {
        // Does not work for unchecked anymore
        let mut t = BoundedInt::<{ Included(&isize::MIN) }, { Excluded(&isize::MAX) }>(0);
        b.iter(|| {
            for i in -2000_isize..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.try_set_fn(&|v| v + i).ok();
            }
        });
        /*let mut t = BoundedInt::<{ isize::MIN }>(isize::MIN);
        b.iter(|| {
            for i in -2000..2000 {
                // 3.4 ns/iter
                let _ = t.try_set_fn(&|v| v + i).ok();
            }
        });*/
    }
}
