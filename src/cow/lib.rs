#[crate_id = "github.com/csherratt/cow-rs#cow:0.1"];
#[comment = "An OpenGL function loader."];
#[license = "ASL2"];
#[crate_type = "lib"];

use std::cast;
use std::clone::Clone;
use std::kinds::Send;
use std::ops::Drop;
use std::ptr::RawPtr;
use std::sync::atomics::{AtomicUint, SeqCst, Relaxed, Acquire};
use std::vec;

pub mod btree;
pub mod join;

/// An atomically reference counted pointer.
///
/// Enforces no shared-memory safety.
#[unsafe_no_drop_flag]
pub struct UnsafeArc<T> {
    priv data: *mut ArcData<T>,
}

struct ArcData<T> {
    count: AtomicUint,
    data: T,
}

unsafe fn new_inner<T: Send>(data: T, refcount: uint) -> *mut ArcData<T> {
    let data = ~ArcData { count: AtomicUint::new(refcount), data: data };
    cast::transmute(data)
}

impl<T: Send> UnsafeArc<T> {
    /// Creates a new `UnsafeArc` which wraps the given data.
    pub fn new(data: T) -> UnsafeArc<T> {
        unsafe { UnsafeArc { data: new_inner(data, 1) } }
    }

    /// As new(), but returns an extra pre-cloned handle.
    pub fn new2(data: T) -> (UnsafeArc<T>, UnsafeArc<T>) {
        unsafe {
            let ptr = new_inner(data, 2);
            (UnsafeArc { data: ptr }, UnsafeArc { data: ptr })
        }
    }

    /// As new(), but returns a vector of as many pre-cloned handles as
    /// requested.
    pub fn newN(data: T, num_handles: uint) -> ~[UnsafeArc<T>] {
        unsafe {
            if num_handles == 0 {
                ~[] // need to free data here
            } else {
                let ptr = new_inner(data, num_handles);
                vec::from_fn(num_handles, |_| UnsafeArc { data: ptr })
            }
        }
    }

    /// Gets a pointer to the inner shared data. Note that care must be taken to
    /// ensure that the outer `UnsafeArc` does not fall out of scope while this
    /// pointer is in use, otherwise it could possibly contain a use-after-free.
    #[inline]
    pub fn get(&self) -> *mut T {
        unsafe {
            assert!((*self.data).count.load(Relaxed) > 0);
            return &mut (*self.data).data as *mut T;
        }
    }

    /// Gets an immutable pointer to the inner shared data. This has the same
    /// caveats as the `get` method.
    #[inline]
    pub fn get_immut(&self) -> *T {
        unsafe {
            assert!((*self.data).count.load(Relaxed) > 0);
            return &(*self.data).data as *T;
        }
    }

    /// checks if this is the only reference to the arc protected data
    #[inline]
    pub fn is_owned(&self) -> bool {
        unsafe {
            (*self.data).count.load(Relaxed) == 1
        }
    }
}

impl<T: Send> Clone for UnsafeArc<T> {
    fn clone(&self) -> UnsafeArc<T> {
        unsafe {
            // This barrier might be unnecessary, but I'm not sure...
            let old_count = (*self.data).count.fetch_add(1, Acquire);
            assert!(old_count >= 1);
            return UnsafeArc { data: self.data };
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for UnsafeArc<T>{
    fn drop(&mut self) {
        unsafe {
            // Happens when destructing an unwrapper's handle and from
            // `#[unsafe_no_drop_flag]`
            if self.data.is_null() {
                return
            }
            // Must be acquire+release, not just release, to make sure this
            // doesn't get reordered to after the unwrapper pointer load.
            let old_count = (*self.data).count.fetch_sub(1, SeqCst);
            assert!(old_count >= 1);
            if old_count == 1 {
                let _: ~ArcData<T> = cast::transmute(self.data);
            }
        }
    }
}


/****************************************************************************
 * Copy-on-write Arc
 ****************************************************************************/

pub struct CowArc<T> { priv x: UnsafeArc<T> }

/// A Copy-on-write Arc functions the same way as an `arc` except it allows
/// mutation of the contents if there is only a single reference to
/// the data. If there are multiple references the data is automatically
/// cloned and the task modifies the cloned data in place of the shared data.
impl<T:Clone+Send+Freeze> CowArc<T> {
    /// Create a copy-on-write atomically reference counted wrapper
    #[inline]
    pub fn new(data: T) -> CowArc<T> {
        CowArc { x: UnsafeArc::new(data) }
    }

    #[inline]
    pub fn get<'a>(&'a self) -> &'a T {
        unsafe { &*self.x.get_immut() }
    }

    /// get a mutable reference to the contents. If there are more then one
    /// reference to the contents of the `CowArc` will be cloned
    /// and this reference updated to point to the cloned data.
    #[inline]
    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        if !self.x.is_owned() {
            *self = CowArc::new(self.get().clone())
        }
        unsafe { &mut *self.x.get() }
    }
}

impl<T:Clone+Send+Freeze> Clone for CowArc<T> {
    /// Duplicate a Copy-on-write Arc. See arc::clone for more details.
    #[inline]
    fn clone(&self) -> CowArc<T> {
        CowArc { x: self.x.clone() }
    }
}