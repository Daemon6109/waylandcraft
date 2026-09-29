use crate::{
    WLCState,
    bridge::{
        BridgeError,
        java_types::*,
        utils::with_env,
    },
};
use jni::{
    Env, objects::Global,
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::with_states,
};
use std::sync::Arc;

struct MySurfaceInner(Global<WLCSurface<'static>>);
type MySurface = Arc<MySurfaceInner>;

pub fn get_surface_data(
    surface: &WlSurface
) -> MySurface {
    with_states(surface, |data| {
        data.data_map
            .get::<MySurface>()
            .unwrap()
            .clone()
    })
}

pub fn new_surface(state: &mut WLCState, surface: &WlSurface) {
    with_env(|env| _new_surface(env, state, surface));
}

fn _new_surface<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    surface: &WlSurface
) -> Result<(), BridgeError> {
    // Create java surface and insert global reference into wl_surface data
    let jsurface = WLCSurface::new(env)?;
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
    let jsurface = &get_surface_data(surface).0;
    state.bridge.java.delete_surface(env, jsurface)?;

    Ok(())
}
