/* This file contains the horrible crimes of the bridge */
// nothing here is safe. feel free to despair

use jni::sys::jlong;

pub fn jptr_to_ref<T>(ptr: jlong) -> Option<&'static T> {
    let ptr = (ptr as usize) as *const T;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*ptr })
    }
}

pub fn jptr_to_mut<T>(ptr: jlong) -> Option<&'static mut T> {
    let ptr = (ptr as usize) as *mut T;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &mut *ptr })
    }
}

#[macro_export]
macro_rules! jptr_to_instance {
    ($jptr:expr) => {
        match jptr_to_mut::<WaylandCraft>($jptr) {
            None => Err(BridgeError::NullInstancePtr),
            Some(wlc) => Ok(wlc),
        }
    };
}
pub use jptr_to_instance;

#[macro_export]
macro_rules! jptr_to_wl_surface {
    ($jptr:expr, $location:literal) => {
        match jptr_to_mut::<WlSurface>($jptr) {
            None => Err(BridgeError::NullSurfacePtr($location)),
            Some(wl_surface) => Ok(wl_surface),
        }
    };
}
pub use jptr_to_wl_surface;

#[macro_export]
macro_rules! jptr_to_toplevel {
    ($jptr:expr, $location:literal) => {
        match jptr_to_mut::<ToplevelSurface>($jptr) {
            None => Err(BridgeError::NullToplevelPtr($location)),
            Some(toplevel) => Ok(toplevel),
        }
    };
}
pub use jptr_to_toplevel;

#[macro_export]
macro_rules! jptr_to_popup {
    ($jptr:expr, $location:literal) => {
        match jptr_to_mut::<PopupSurface>($jptr) {
            None => Err(BridgeError::NullPopupPtr($location)),
            Some(popup) => Ok(popup),
        }
    };
}
pub use jptr_to_popup;

// Get or insert an element and return its handle
pub fn insert_get_handle<T>(vec: &mut Vec<Box<T>>, elem: &T) -> jlong
where
    T: Clone + PartialEq,
{
    if !vec.iter().any(|b| **b == *elem) {
        vec.push(Box::new(elem.clone()));
    }

    let ptr: &mut Box<T> = vec.iter_mut().find(|r| ***r == *elem).unwrap();
    let ptr: *mut T = &raw mut **ptr;
    (ptr as usize) as jlong
}

// Get an element and return its handle
// Element has to be in the list, otherwise this functions panics
pub fn get_handle<T>(vec: &[Box<T>], elem: &T) -> jlong
where
    T: Clone + PartialEq,
{
    let ptr: &T = vec.iter().find(|r| ***r == *elem).unwrap();
    ((ptr as *const T) as usize) as jlong
}

// Get an element and return its handle
pub fn get_handle_safe<T>(vec: &[Box<T>], elem: &T) -> Option<jlong>
where
    T: Clone + PartialEq,
{
    let ptr: Option<&T> = vec.iter().find(|r| ***r == *elem).map(|v| &**v);
    ptr.map(|p| ((p as *const T) as usize) as jlong)
}

// Insert all elements that aren't in the list already
pub fn insert_all<T>(vec: &mut Vec<Box<T>>, elems: &[T])
where
    T: Clone + PartialEq,
{
    for elem in elems {
        if !vec.iter().any(|b| **b == *elem) {
            vec.push(Box::new(elem.clone()));
        }
    }
}

// Get handles of all elements in the list
pub fn get_all_handles<T>(vec: &mut [Box<T>]) -> Vec<jlong>
where
    T: Clone + PartialEq,
{
    vec.iter_mut()
        .map(|r| ((&mut **r) as *mut T) as usize as jlong)
        .collect()
}

// Remove element from list and free it
pub fn remove_element<T>(vec: &mut Vec<Box<T>>, handle: jlong)
where
    T: Clone + PartialEq,
{
    let ptr: *mut T = (handle as usize) as *mut T;
    let elem: &mut T = unsafe { &mut *ptr };
    vec.retain(|e| **e != *elem);
}
