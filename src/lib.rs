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
    InvalidConversion,
}

macro_rules! bound_idx {
    ($value_name:ident, $value_type:ty) => {
        pub trait BoundedValueTrait {
            // This leaks value_type so not possible to define for two types using same trait name in upper module scope. not fastest to parse either if impl for each
            // new cant be here either as it references mod_name::Value which is const for all
            // I don't think it's good basis for lib. The try_new is good addition tho to test initial values
            //fn get_bounds() -> Range<$value_type>;
            fn is_valid(&self) -> bool;
            fn try_get<T: TryFrom<$value_type>>(&self) -> Result<T, BoundErr> {
                if self.is_valid() {
                    /// SAFETY: the validity is checked before calling
                    let value = unsafe { self.get_unchecked() };
                    match T::try_from(value) {
                        Ok(v) => Ok(v),
                        Err(_) => Err(BoundErr::InvalidConversion)
                    }
                } else {
                    Err(Invalid)
                }
            }
            fn try_set(&mut self, new_value: $value_type) -> Result<(), crate::BoundErr>;
            fn try_set_fn(&mut self, set_fn: &impl Fn(&mut Self)) -> Result<(), crate::BoundErr>;
            fn invalidate(&mut self);
            unsafe fn get_unchecked(&self) -> $value_type;
            unsafe fn set_unchecked(&mut self, new_value: $value_type);
            unsafe fn set_fn_unchecked(&mut self, set_fn: &impl Fn(&mut Self));
        }
        pub mod $value_name {
                use crate::BoundedValueTrait;
                use crate::BoundErr;
                use crate::BoundErr::*;
                use std::ops::Range;
                pub struct Value <
                const UPPER: $value_type,
                const LOWER: $value_type> ($value_type);
                pub fn try_new<const LOWER: $value_type, const UPPER: $value_type>(init_val: $value_type) -> Result<Value<UPPER, LOWER>,BoundErr> { Ok(Value::<UPPER, LOWER>(init_val)) }
                impl<const UPPER: $value_type, const LOWER: $value_type> Value<UPPER,LOWER> {
                    fn get_bounds() -> Range<$value_type> { Range { start: LOWER, end: UPPER } }
                }
                impl<const UPPER: $value_type, const LOWER: $value_type> BoundedValueTrait
                for Value<UPPER, LOWER> {
                    fn is_valid(&self) -> bool {
                        Self::get_bounds().contains(&self.0)
                    }
                    fn try_set(&mut self, new_value: $value_type) -> Result<(), BoundErr> {
                        if self.is_valid() {
                            if Self::get_bounds().contains(&new_value) {
                                self.0 = new_value as $value_type;
                                Ok(())
                            } else {
                                Err(BoundError)
                            }
                        } else {
                            Err(Invalid)
                        }
                    }
                    /// Set function uses provided api and can use unsafe methods as well.
                    /// This function checks the if the end result is valid and return Result regarding the validity of contained value.
                    /// It is not known if function changed contained value
                    /// Should invalidate be called in error cases to or return out of bounds?
                    /// If not invalidated it could be possible to set out-of bounds values in unsafe
                    fn try_set_fn(&mut self,set_fn: &impl Fn(&mut Self)) -> Result<(), BoundErr> {
                       set_fn(self);
                        if self.is_valid() {
                            Ok(())
                        } else {
                            Err(Invalid)
                        }
                    }
                    fn invalidate(&mut self) {
                        // is_valid() is now false
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
bound_idx!(B, i16);

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
        let t = B::try_new::<0i16,200i16>(190i16).ok().unwrap();
        assert!(t.is_valid());
        assert!(!t.try_get::<i8>().is_ok());

        /*let t = B::Value::<{ 5 }, { 2isize }>(5);
        assert!(!t.is_valid());
        let t = B::Value::<{ isize::MAX }>(5);
        assert!(t.is_valid());
        assert_eq!(
            size_of::<B::Value::<{ isize::MAX }, { isize::MIN }>>(),
            size_of::<isize>()
        );*/
    }
/*
    #[bench]
    fn it_folds_unchecked_set_fn(b: &mut Bencher) {
        let mut t = B::Value::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn_unchecked(&|mut v| v.set_unchecked(i as isize));
            }
        });
    }

    #[bench]
    fn it_folds_unchecked_set_fn_bb(b: &mut Bencher) {
        let mut t = B::Value::<{ isize::MIN }, { isize::MAX }>(0isize);
        b.iter(|| unsafe {
            for i in -2000..2000 {
                // Optimized out or 1.9 ns/iter with black box
                let _ = t.set_fn_unchecked(&|mut v| black_box(v.set_unchecked(i)));
            }
        });
    }

    #[bench]
    fn it_folds_try_set_fn_bb(b: &mut Bencher) {
        let mut t = B::Value::<{ isize::MIN }, { isize::MAX }>(0isize);
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
        let mut t = B::Value::<{ isize::MIN }, { isize::MAX }>(0isize);
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
*/

}
