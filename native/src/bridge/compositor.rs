use crate::{
    WLCState,
    bridge::{
        BridgeError,
        java_types::*,
        utils::with_env,
    },
    utils::get_time,
};
use jni::{
    Env,
    objects::Global,
    sys::jlong,
};
use smithay::{
    reexports::wayland_server::{
        Resource, Weak,
        protocol::wl_surface::WlSurface,
    },
    wayland::compositor::{
        with_states, SurfaceAttributes
    },
};
use std::sync::Arc;
use std::ops::DerefMut;

pub struct MySurfaceInner(pub Global<WLCSurface<'static>>);
type MySurface = Arc<MySurfaceInner>;

pub fn surface_user_data(
    surface: &WlSurface
) -> MySurface {
    with_states(surface, |data| {
        data.data_map
            .get::<MySurface>()
            .unwrap()
            .clone()
    })
}

#[macro_export]
macro_rules! get_java_surface {
    ($surface:expr) => {
        (&surface_user_data($surface).0)
    };
}

pub fn new_surface(state: &mut WLCState, surface: &WlSurface) {
    with_env(|env| _new_surface(env, state, surface));
}

fn _new_surface<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    surface: &WlSurface
) -> Result<(), BridgeError> {
    // Create handle from boxed WlSurface weak reference
    let weak: Weak<WlSurface> = surface.downgrade();
    let weak = Box::new(weak);
    let ptr = (Box::into_raw(weak) as usize) as jlong;

    // Create java surface and insert global reference into wl_surface data
    let jsurface = WLCSurface::new(env, ptr)?;
    let jsurface_ref = env.new_global_ref(&jsurface)?;
    let my_surface = Arc::new(MySurfaceInner(jsurface_ref));

    with_states(surface, |data| {
        data.data_map.insert_if_missing(|| my_surface);
    });

    // Tell the java code about the new surface
    state.bridge.java.add_surface(env, jsurface)?;

    Ok(())
}

pub fn surface_destroyed(state: &mut WLCState, surface: &WlSurface) {
    with_env(|env| _surface_destroyed(env, state, surface));
}

fn _surface_destroyed<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    surface: &WlSurface
) -> Result<(), BridgeError> {
    // Remove surface in java bridge code
    let jsurface = get_java_surface!(surface);
    state.bridge.java.delete_surface(env, jsurface)?;

    // Remove and destroy its handle
    let ptr = (jsurface.handle(env)? as usize) as *mut Weak<WlSurface>;
    let ptr = unsafe { Box::from_raw(ptr) };
    drop(ptr);
    jsurface.set_handle(env, 0)?;

    // The java global reference will get dropped with the surface

    Ok(())
}

fn surface_from_java_nullable<'local>(
    env: &mut Env<'local>,
    jsurface: WLCSurface<'local>,
) -> Result<Option<WlSurface>, BridgeError> {
    if jsurface.is_null() {
        return Ok(None);
    }

    let ptr = (jsurface.handle(env)? as usize) as *mut Weak<WlSurface>;
    if ptr.is_null() {
        return Err(BridgeError::SurfaceGone);
    }

    let weak = unsafe { &mut *ptr };
    let surface = weak.upgrade().map_err(|_| BridgeError::SurfaceGone)?;

    Ok(Some(surface))
}

fn surface_from_java<'local>(
    env: &mut Env<'local>,
    jsurface: WLCSurface<'local>,
) -> Result<WlSurface, BridgeError> {
    surface_from_java_nullable(env, jsurface)?.ok_or(BridgeError::SurfaceNull)
}

pub fn send_frame<'local>(
    env: &mut Env<'local>,
    jsurface: WLCSurface<'local>,
) -> Result<(), BridgeError> {
    let surface = surface_from_java(env, jsurface)?;

    with_states(&surface, |data| {
        let mut attr_guard = data.cached_state.get::<SurfaceAttributes>();
        let attr = attr_guard.deref_mut().current();
        for c in attr.frame_callbacks.drain(..) {
            c.done(get_time());
        }
    });

    Ok(())
}
