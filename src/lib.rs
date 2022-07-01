#![feature(test)]
extern crate test;

use std::borrow::Borrow;
use std::collections::Bound;
use std::env::Args;
use std::ops::{Range, RangeBounds, RangeInclusive, RangeToInclusive};
use std::ops::Bound::{Excluded, Included};

use crate::BoundedIntErr::{BoundError, Invalid};

macro_rules! BoundIdx {
    ($value_type:ty) => {
        pub trait BoundedIdxTrait {
            fn is_valid(&self) -> bool;
            fn get_bounds() -> Range<$value_type>;
            fn try_get(&self) -> Result<$value_type, BoundedIntErr>;
            fn try_set(&mut self, new_value: $value_type) -> Result<$value_type, BoundedIntErr>;
            fn try_set_fn(&mut self, set_with_fn: &impl Fn(&mut Self)) -> Result<$value_type, BoundedIntErr>;
            fn invalidate(&mut self);
            unsafe fn set(&mut self, new_value: $value_type) -> $value_type;
            unsafe fn set_fn(&mut self, set_with_fn: &impl Fn($value_type) -> $value_type) -> $value_type;
            unsafe fn get(&self) -> $value_type;
        }
        pub struct BoundIdx<
            const LOWER: $value_type = { <$value_type>::MIN },
            const UPPER: $value_type = { <$value_type>::MAX - 1 }> ($value_type);
        impl<const LOWER: $value_type, const UPPER: $value_type> RangeBounds<$value_type> for BoundIdx<LOWER, UPPER> {
            fn start_bound(&self) -> Bound<&$value_type> {
                Included(&LOWER)
            }
            fn end_bound(&self) -> Bound<&$value_type> {
                Included(&UPPER)
            }
        }
        impl<const LOWER: $value_type, const UPPER: $value_type> BoundedIdxTrait for BoundIdx<LOWER, UPPER> {
            // Boilerplate to Self::RangeBounds::contains
            fn is_valid(&self) -> bool {
                self.contains(&self.0)
            }
            fn get_bounds() -> Range<$value_type> {
                Range { start: LOWER, end: UPPER }
            }
            fn try_get(&self) -> Result<$value_type, BoundedIntErr> {
                if self.is_valid() {
                    Ok(self.0)
                } else {
                    Err(Invalid)
                }
            }
            fn try_set(&mut self, new_value: $value_type) -> Result<$value_type, BoundedIntErr> {
                if self.contains(&new_value) {
                    self.0 = new_value as $value_type;
                    Ok(self.0)
                } else {
                    Err(BoundError)
                }
            }
            fn try_set_fn(&mut self, set_with_fn: &impl Fn(&mut Self)) -> Result<$value_type, BoundedIntErr> {
                set_with_fn(self);
                if self.is_valid() {
                    Ok(self.0)
                } else {
                    Err(Invalid)
                }
            }
            fn invalidate(&mut self) {
                // is_valid() is now false
                self.0 = UPPER;
            }
            unsafe fn set(&mut self, new_value: $value_type) -> $value_type {
                self.0 = new_value;
                self.0
            }
            unsafe fn set_fn(&mut self, set_with_fn: &impl Fn($value_type) -> $value_type) -> $value_type {
                self.0 = set_with_fn(self.0);
                self.0
            }
            unsafe fn get(&self) -> $value_type {
                self.0
            }
        }
    }
}

/// Valid range Not Inclusive meaning that MAX value is invalid
/// eg. (0..4) is actually valid for values 0,1,2,3 and 4 is not valid.
pub struct BoundedInt<
    const LOWER: isize = { isize::MIN },
    const UPPER: isize = { isize::MAX }> (isize);

pub enum BoundedIntErr {
    BoundError,
    Invalid,
}

impl<const LOWER: isize, const UPPER: isize> RangeBounds<isize> for BoundedInt<LOWER, UPPER> {
    fn start_bound(&self) -> Bound<&isize> {
        Included(&LOWER)
    }
    fn end_bound(&self) -> Bound<&isize> {
        Excluded(&UPPER)
    }
}

impl<const LOWER: isize, const UPPER: isize> BoundedIntTrait for BoundedInt<LOWER, UPPER> {
    // Boilerplate to Self::RangeBounds::contains
    fn is_valid(&self) -> bool {
        self.contains(&self.0)
    }
    fn get_bounds() -> Range<isize> {
        Range { start: LOWER, end: UPPER }
    }
    fn try_get(&self) -> Result<isize, BoundedIntErr> {
        if self.is_valid() {
            Ok(self.0)
        } else {
            Err(Invalid)
        }
    }
    fn try_set(&mut self, new_value: isize) -> Result<isize, BoundedIntErr> {
        if self.contains(&new_value) {
            self.0 = new_value;
            Ok(self.0)
        } else {
            Err(BoundError)
        }
    }
    fn try_set_fn(&mut self, set_with_fn: &impl Fn(&mut Self)) -> Result<isize, BoundedIntErr> {
        set_with_fn(self);
        if self.is_valid() {
            Ok(self.0)
        } else {
            Err(Invalid)
        }
    }

    fn invalidate(&mut self) {
        // is_valid() is now false
        self.0 = UPPER;
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
    fn try_set_fn(&mut self, set_with_fn: &impl Fn(&mut Self)) -> Result<isize, BoundedIntErr>;
    fn invalidate(&mut self);
}

BoundIdx!(i8);

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
        let t = BoundedInt::<{ isize::MIN }, { 5isize }>(5);
        assert!(!t.is_valid());
        let t = BoundedInt::<{ isize::MIN }, { 10isize }>(5);
        assert!(t.is_valid());
        assert_eq!(size_of::<BoundedInt::<{ isize::MIN }, { isize::MAX }>>(), size_of::<isize>());
    }

    #[bench]
    fn it_folds_unchecked_set_fn(b: &mut Bencher) {
        let mut t = BoundedInt(0);
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
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn(&|v| black_box(v + i));
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn_bb(b: &mut Bencher) {
        let mut t = BoundedInt::<{ isize::MIN }>(isize::MIN);
        b.iter(|| {
            for i in -2000..2000 {
                // 1.8 ns/iter with blackbox
                //let _ = t.try_set_fn(&|v| black_box(v + i));
                let _ = t.try_set_fn(&|mut v| { v.try_set(black_box(i)).ok(); });
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn(b: &mut Bencher) {
        // Does not work for unchecked anymore
        let mut t = BoundedInt::<{ isize::MIN }, { isize::MAX }>(0);
        b.iter(|| {
            for i in -2000_isize..2000 {
                // Optimized out or 1.9 ns/iter with black box
                //let _ = t.try_set_fn(&|v| v + i).ok();
                let _ = t.try_set_fn(&|mut v| { v.try_set(i).ok(); });
            }
        });
    }
}
