use crate::shim::atomic::Ordering;
use crate::shim::atomic::{
    AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16,
    AtomicU32, AtomicU64, AtomicUsize,
};

/// Trait for types that support atomic operations
///
/// 支持原子操作的类型 trait
///
/// This trait abstracts atomic operations to allow generic implementation
/// over different atomic types (AtomicU8, AtomicU64, AtomicUsize, etc.)
///
/// 此 trait 抽象了原子操作，允许在不同原子类型上实现泛型
/// (AtomicU8, AtomicU64, AtomicUsize 等)
pub trait AtomicElement: Send + Sync {
    /// The underlying primitive type
    ///
    /// 底层原始类型
    type Primitive: Copy;

    /// Load the value with specified ordering
    ///
    /// 使用指定的内存顺序加载值
    fn load(&self, order: Ordering) -> Self::Primitive;

    /// Store a value with specified ordering
    ///
    /// 使用指定的内存顺序存储值
    fn store(&self, val: Self::Primitive, order: Ordering);

    /// Swap the value and return the old value
    ///
    /// 交换值并返回旧值
    fn swap(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive;
}

macro_rules! impl_atomic_element {
    ($atomic:ty, $primitive:ty) => {
        impl AtomicElement for $atomic {
            type Primitive = $primitive;

            #[inline(always)]
            fn load(&self, order: Ordering) -> Self::Primitive {
                self.load(order)
            }

            #[inline(always)]
            fn store(&self, val: Self::Primitive, order: Ordering) {
                self.store(val, order);
            }

            #[inline(always)]
            fn swap(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                self.swap(val, order)
            }
        }
    };
}

impl_atomic_element!(AtomicU8, u8);
impl_atomic_element!(AtomicU16, u16);
impl_atomic_element!(AtomicU32, u32);
impl_atomic_element!(AtomicU64, u64);
impl_atomic_element!(AtomicUsize, usize);
impl_atomic_element!(AtomicI8, i8);
impl_atomic_element!(AtomicI16, i16);
impl_atomic_element!(AtomicI32, i32);
impl_atomic_element!(AtomicI64, i64);
impl_atomic_element!(AtomicIsize, isize);
impl_atomic_element!(AtomicBool, bool);

#[cfg(any(feature = "loom", feature = "portable-atomic"))]
macro_rules! impl_core_atomic_element {
    ($atomic:ty, $primitive:ty) => {
        impl AtomicElement for $atomic {
            type Primitive = $primitive;

            #[inline(always)]
            fn load(&self, order: Ordering) -> Self::Primitive {
                let core_order = match order {
                    Ordering::Relaxed => core::sync::atomic::Ordering::Relaxed,
                    Ordering::Release => core::sync::atomic::Ordering::Release,
                    Ordering::Acquire => core::sync::atomic::Ordering::Acquire,
                    Ordering::AcqRel => core::sync::atomic::Ordering::AcqRel,
                    Ordering::SeqCst => core::sync::atomic::Ordering::SeqCst,
                    _ => core::sync::atomic::Ordering::SeqCst,
                };
                self.load(core_order)
            }

            #[inline(always)]
            fn store(&self, val: Self::Primitive, order: Ordering) {
                let core_order = match order {
                    Ordering::Relaxed => core::sync::atomic::Ordering::Relaxed,
                    Ordering::Release => core::sync::atomic::Ordering::Release,
                    Ordering::Acquire => core::sync::atomic::Ordering::Acquire,
                    Ordering::AcqRel => core::sync::atomic::Ordering::AcqRel,
                    Ordering::SeqCst => core::sync::atomic::Ordering::SeqCst,
                    _ => core::sync::atomic::Ordering::SeqCst,
                };
                self.store(val, core_order);
            }

            #[inline(always)]
            fn swap(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                let core_order = match order {
                    Ordering::Relaxed => core::sync::atomic::Ordering::Relaxed,
                    Ordering::Release => core::sync::atomic::Ordering::Release,
                    Ordering::Acquire => core::sync::atomic::Ordering::Acquire,
                    Ordering::AcqRel => core::sync::atomic::Ordering::AcqRel,
                    Ordering::SeqCst => core::sync::atomic::Ordering::SeqCst,
                    _ => core::sync::atomic::Ordering::SeqCst,
                };
                self.swap(val, core_order)
            }
        }
    };
}

#[cfg(any(feature = "loom", feature = "portable-atomic"))]
mod core_atomic_impls {
    use super::*;

    #[cfg(target_has_atomic = "8")]
    impl_core_atomic_element!(core::sync::atomic::AtomicBool, bool);
    #[cfg(target_has_atomic = "8")]
    impl_core_atomic_element!(core::sync::atomic::AtomicU8, u8);
    #[cfg(target_has_atomic = "8")]
    impl_core_atomic_element!(core::sync::atomic::AtomicI8, i8);

    #[cfg(target_has_atomic = "16")]
    impl_core_atomic_element!(core::sync::atomic::AtomicU16, u16);
    #[cfg(target_has_atomic = "16")]
    impl_core_atomic_element!(core::sync::atomic::AtomicI16, i16);

    #[cfg(target_has_atomic = "32")]
    impl_core_atomic_element!(core::sync::atomic::AtomicU32, u32);
    #[cfg(target_has_atomic = "32")]
    impl_core_atomic_element!(core::sync::atomic::AtomicI32, i32);

    #[cfg(target_has_atomic = "64")]
    impl_core_atomic_element!(core::sync::atomic::AtomicU64, u64);
    #[cfg(target_has_atomic = "64")]
    impl_core_atomic_element!(core::sync::atomic::AtomicI64, i64);

    #[cfg(target_has_atomic = "ptr")]
    impl_core_atomic_element!(core::sync::atomic::AtomicUsize, usize);
    #[cfg(target_has_atomic = "ptr")]
    impl_core_atomic_element!(core::sync::atomic::AtomicIsize, isize);
}

/// Trait for atomic numeric operations on ring buffer elements
///
/// 环形缓冲区元素的原子数值操作 trait
///
/// This trait provides atomic arithmetic operations for numeric atomic types.
///
/// 此 trait 为数值原子类型提供原子算术操作。
pub trait AtomicNumeric: AtomicElement {
    /// Atomically add to the value at the given offset from read position
    ///
    /// 原子地将值加到从读位置开始的给定偏移处
    fn fetch_add(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive;

    /// Atomically subtract from the value at the given offset from read position
    ///
    /// 原子地从给定偏移处的值减去
    fn fetch_sub(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive;
}

macro_rules! impl_atomic_numeric {
    ($atomic:ty, $primitive:ty) => {
        impl AtomicNumeric for $atomic {
            #[inline]
            fn fetch_add(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                self.fetch_add(val, order)
            }

            #[inline]
            fn fetch_sub(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                self.fetch_sub(val, order)
            }
        }
    };
}

impl_atomic_numeric!(AtomicU8, u8);
impl_atomic_numeric!(AtomicU16, u16);
impl_atomic_numeric!(AtomicU32, u32);
impl_atomic_numeric!(AtomicU64, u64);
impl_atomic_numeric!(AtomicUsize, usize);
impl_atomic_numeric!(AtomicI8, i8);
impl_atomic_numeric!(AtomicI16, i16);
impl_atomic_numeric!(AtomicI32, i32);
impl_atomic_numeric!(AtomicI64, i64);
impl_atomic_numeric!(AtomicIsize, isize);

#[cfg(any(feature = "loom", feature = "portable-atomic"))]
macro_rules! impl_core_atomic_numeric {
    ($atomic:ty) => {
        impl AtomicNumeric for $atomic {
            #[inline]
            fn fetch_add(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                let core_order = match order {
                    Ordering::Relaxed => core::sync::atomic::Ordering::Relaxed,
                    Ordering::Release => core::sync::atomic::Ordering::Release,
                    Ordering::Acquire => core::sync::atomic::Ordering::Acquire,
                    Ordering::AcqRel => core::sync::atomic::Ordering::AcqRel,
                    Ordering::SeqCst => core::sync::atomic::Ordering::SeqCst,
                    _ => core::sync::atomic::Ordering::SeqCst,
                };
                self.fetch_add(val, core_order)
            }

            #[inline]
            fn fetch_sub(&self, val: Self::Primitive, order: Ordering) -> Self::Primitive {
                let core_order = match order {
                    Ordering::Relaxed => core::sync::atomic::Ordering::Relaxed,
                    Ordering::Release => core::sync::atomic::Ordering::Release,
                    Ordering::Acquire => core::sync::atomic::Ordering::Acquire,
                    Ordering::AcqRel => core::sync::atomic::Ordering::AcqRel,
                    Ordering::SeqCst => core::sync::atomic::Ordering::SeqCst,
                    _ => core::sync::atomic::Ordering::SeqCst,
                };
                self.fetch_sub(val, core_order)
            }
        }
    };
}

#[cfg(any(feature = "loom", feature = "portable-atomic"))]
mod core_atomic_numeric_impls {
    use super::*;

    #[cfg(target_has_atomic = "8")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicU8);
    #[cfg(target_has_atomic = "8")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicI8);

    #[cfg(target_has_atomic = "16")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicU16);
    #[cfg(target_has_atomic = "16")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicI16);

    #[cfg(target_has_atomic = "32")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicU32);
    #[cfg(target_has_atomic = "32")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicI32);

    #[cfg(target_has_atomic = "64")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicU64);
    #[cfg(target_has_atomic = "64")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicI64);

    #[cfg(target_has_atomic = "ptr")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicUsize);
    #[cfg(target_has_atomic = "ptr")]
    impl_core_atomic_numeric!(core::sync::atomic::AtomicIsize);
}
