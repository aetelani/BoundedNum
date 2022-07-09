#![feature(test)]
extern crate test;

use std::borrow::Borrow;
use std::collections::Bound;
use std::env::Args;
use std::ops::{Range, RangeBounds, RangeInclusive, RangeToInclusive};
use std::ops::Bound::{Excluded, Included};
use paste::paste;
use crate::BoundErr::{BoundError, Invalid};

pub enum BoundErr {
    BoundError,
    Invalid,
}

macro_rules! bound_idx {
    ($value_name:ident, $value_type:ty) => {
        paste! {
            pub trait [<$value_name Trait>] {
                fn valid(&self) -> bool;
                fn get_bounds<const LOWER: $value_type, const UPPER: $value_type>() -> Range<$value_type> { Range { start: LOWER, end: UPPER } }
                fn try_get(&self) -> Result<$value_type, BoundErr>;
                fn try_set(&mut self, new_value: $value_type) -> Result<(), BoundErr>;
                fn try_set_fn(&mut self, set_fn: &impl Fn(&mut Self)) -> Result<(), BoundErr>;
                fn invalidate(&mut self);
                unsafe fn get_unchecked(&self) -> $value_type;
                unsafe fn set_unchecked(&mut self, new_value: $value_type);
                unsafe fn set_fn_unchecked(&mut self, set_fn: &impl Fn(&mut Self));
            }
            pub struct [< $value_name $value_type >] <
                const UPPER: $value_type = { <$value_type>::MAX },
                const LOWER: $value_type = { <$value_type>::MIN }> ($value_type);
            impl<const UPPER: $value_type, const LOWER: $value_type> [<$value_name Trait>]
            for [< $value_name $value_type >]<UPPER, LOWER> {
                fn valid(&self) -> bool {
                    Self::get_bounds::<LOWER, UPPER>().contains(&self.0)
                }
                fn try_get(&self) -> Result<$value_type, BoundErr> {
                    if self.valid() {
                        Ok(self.0)
                    } else {
                        Err(Invalid)
                    }
                }
                fn try_set(&mut self, new_value: $value_type) -> Result<(), BoundErr> {
                    if Self::get_bounds::<LOWER, UPPER>().contains(&new_value) {
                        self.0 = new_value as $value_type;
                        Ok(())
                    } else {
                        Err(BoundError)
                    }
                }
                fn try_set_fn(&mut self,set_fn: &impl Fn(&mut Self)) -> Result<(), BoundErr> {
                   set_fn(self);
                    if self.valid() {
                        Ok(())
                    } else {
                        Err(Invalid)
                    }
                }
                fn invalidate(&mut self) {
                    // valid() is now false
                    self.0 = UPPER;
                }
                unsafe fn set_unchecked(&mut self, new_value: $value_type) {
                    self.0 = new_value;
                }
                unsafe fn set_fn_unchecked(&mut self,set_fn: &impl Fn(&mut Self)) {
                    set_fn(self);
                }
                unsafe fn get_unchecked(&self) -> $value_type {
                    self.0
                }
            }
        }
    }
}
bound_idx!(B, isize);

#[cfg(test)]
mod tests {
    use std::collections::Bound::Included;
    use std::mem::size_of;
    use std::ops::Add;
    use std::ops::Bound::Excluded;
    use test::{Bencher, black_box};

    use crate::*;

    #[test]
    fn it_jiggles() {
        let t = Bisize::<6>(5i8 as isize);
        assert!(t.valid());

        let t = Bisize::<{ 5 }, { 2isize }>(5);
        assert!(!t.valid());
        let t = Bisize::<{ isize::MAX }>(5);
        assert!(t.valid());
        assert_eq!(
            size_of::<Bisize::<{ isize::MAX }, { isize::MIN }>>(),
            size_of::<isize>()
        );
    }

    #[bench]
    fn it_folds_unchecked_set_fn(b: &mut Bencher) {
        let mut t = Bisize::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn_unchecked(&|mut v| v.set_unchecked(i as isize));
            }
        });
    }

    #[bench]
    fn it_folds_unchecked_set_fn_bb(b: &mut Bencher) {
        let mut t = Bisize::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn_unchecked(&|mut v| black_box(v.set_unchecked(i)));
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn_bb(b: &mut Bencher) {
        let mut t = Bisize::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| {
            for i in -2000isize..2000 {
                // 1.8 ns/iter with blackbox
                //let _ = t.try_set_fn(&|v| black_box(v + i));
                let _ = t.try_set_fn(&|mut v| {
                    v.try_set(black_box(i)).ok();
                });
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn(b: &mut Bencher) {
        // Does not work for unchecked anymore
        let mut t = Bisize::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| {
            for i in -2000_isize..2000 {
                // Optimized out or 1.9 ns/iter with black box
                //let _ = t.try_set_fn(&|v| v + i).ok();
                let _ = t.try_set_fn(&|mut v| {
                    v.try_set(i).ok();
                });
            }
        });
    }
}
