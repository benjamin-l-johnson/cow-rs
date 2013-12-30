#[crate_id = "github.com/csherratt/cow-rs#cow:0.1"];
#[comment = "An OpenGL function loader."];
#[license = "ASL2"];
#[crate_type = "lib"];

use std::sync::atomics::{AtomicUint, SeqCst, Acquire};
use std::cast::{transmute};

pub mod btree;

struct CowCell<T> {
    priv count: AtomicUint,
    priv data: T
}

pub struct Cow<T> {
    priv ptr: *mut CowCell<T>
}

impl<T> CowCell<T>
{
    fn new(data: T) -> ~CowCell<T>
    {
        ~CowCell{
            count: AtomicUint::new(1),
            data: data
        }
    }

    fn inc_ref(&mut self)
    {
        let old_count = self.count.fetch_add(1, Acquire);
        assert!(old_count >= 1);
    }

    fn dec_ref(&mut self) -> bool
    {
        let old_count = self.count.fetch_sub(1, SeqCst);
        old_count == 1
    }
}

impl<T> Cow<T>
{
    pub fn new(data: T) -> Cow<T>
    {
        unsafe {
            Cow {ptr: transmute(CowCell::new(data))}
        }
    }

    unsafe fn inc_ref(&self)
    {
        let cell: &mut CowCell<T> = transmute(self.ptr);
        cell.inc_ref();
    }

    unsafe fn dec_ref(&self) -> bool
    {
        let cell: &mut CowCell<T> = transmute(self.ptr);
        cell.dec_ref()
    }
}

impl<T> Cow<T>
{
    pub fn get<'a>(&'a self) -> &'a T
    {
        unsafe {
            let cell: &CowCell<T> = transmute(self.ptr);
            &cell.data
        }
    }
}

impl<T: Clone> Cow<T>
{
    unsafe fn dup(&mut self) -> Cow<T>
    {
        let cell: &mut CowCell<T> = transmute(self.ptr);
        Cow::new(cell.data.clone())
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T
    {
        unsafe {
            let mut cell: &mut CowCell<T> = transmute(self.ptr);

            // if only one ref it is safe to mutate the
            // content of the cell
            if cell.count.load(Acquire) > 1 {
                // clone the cell content into an new cell
                *self = self.dup();
                cell = transmute(self.ptr);
            }
            
            &mut cell.data
        }
    }
}

impl<T> Clone for Cow<T>
{
    fn clone(&self) -> Cow<T>
    {
        unsafe {
            self.inc_ref();
        }

        Cow {
            ptr: self.ptr
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Cow<T>
{
    fn drop(&mut self)
    {
        unsafe {
            // dec ref, if last ref free data
            if self.dec_ref() {
                let _: ~CowCell<T> = transmute(self.ptr);
            }
        }
    }
}