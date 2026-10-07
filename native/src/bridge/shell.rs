use crate::{
    WLCState,
    bridge::{
        compositor::get_java_surface,
        java_types::{BridgeError, WLCToplevel},
        utils::{jptr_to_instance, with_env},
    },
};
use jni::{
    Env,
    objects::{Global, JClass},
    sys::{jboolean, jint, jlong},
};
use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::{
            self, XdgToplevel,
        },
        wayland_server::{
            Resource, Weak,
            protocol::wl_surface::WlSurface,
        },
    },
    utils::Size,
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, SurfaceCachedState, ToplevelSurface,
        },
    },
};
use std::sync::Arc;

pub struct MyToplevelInner(pub Global<WLCToplevel<'static>>);
type MyToplevel = Arc<MyToplevelInner>;

pub fn toplevel_user_data(
    toplevel: &ToplevelSurface
) -> MyToplevel {
    with_states(toplevel.wl_surface(), |data| {
        data
            .data_map
            .get::<MyToplevel>()
            .unwrap()
            .clone()
    })
}

// Turns &ToplevelSurface into &WLCToplevel
#[macro_export]
macro_rules! get_java_toplevel {
    ($toplevel:expr) => {
        (&$crate::bridge::shell::toplevel_user_data($toplevel).0)
    };
}
pub use get_java_toplevel;

pub fn new_toplevel(state: &mut WLCState, toplevel: &ToplevelSurface) {
    with_env(|env| _new_toplevel(env, state, toplevel))
}

fn _new_toplevel<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    toplevel: &ToplevelSurface
) -> Result<(), BridgeError> {
    // Create handle from boxed XdgToplevel weak reference
    let weak: Weak<XdgToplevel> = toplevel.xdg_toplevel().downgrade();
    let weak = Box::new(weak);
    let ptr = (Box::into_raw(weak) as usize) as jlong;

    let jsurface = get_java_surface!(toplevel.wl_surface());

    // Create java toplevel and insert global reference into toplevel root
    // wl_surface data
    let jtoplevel = WLCToplevel::new(env, ptr, jsurface)?;
    let jtoplevel_ref = env.new_global_ref(&jtoplevel)?;
    let my_toplevel = Arc::new(MyToplevelInner(jtoplevel_ref));

    with_states(toplevel.wl_surface(), |data| {
        data.data_map.insert_if_missing(|| my_toplevel);
    });

    jtoplevel.set_surface(env, jsurface)?;

    // Tell the java code about the new toplevel
    state.bridge.java.add_toplevel(env, jtoplevel)?;

    Ok(())
}

pub fn toplevel_destroyed(state: &mut WLCState, toplevel: &ToplevelSurface) {
    with_env(|env| _toplevel_destroyed(env, state, toplevel))
}

fn _toplevel_destroyed<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    toplevel: &ToplevelSurface
) -> Result<(), BridgeError> {
    // Remove toplevel in java bridge code
    let jtoplevel = get_java_toplevel!(toplevel);
    state.bridge.java.delete_toplevel(env, jtoplevel)?;

    // Remove and destroy its handle
    let ptr = (jtoplevel.handle(env)? as usize) as *mut Weak<XdgToplevel>;
    let ptr = unsafe { Box::from_raw(ptr) };
    drop(ptr);
    jtoplevel.set_handle(env, 0)?;

    // The java global reference will get dropped with the toplevel

    Ok(())
}

pub fn toplevel_for_surface(
    state: &mut WLCState,
    surface: &WlSurface,
) -> Option<ToplevelSurface> {
    state
        .xdg_state
        .toplevel_surfaces()
        .iter()
        .find(|t| t.wl_surface() == surface)
        .cloned()
}

pub fn toplevel_commit<'local>(
    env: &mut Env<'local>,
    _state: &mut WLCState,
    toplevel: &ToplevelSurface,
) -> Result<(), BridgeError> {
    let jtoplevel = get_java_toplevel!(toplevel);
    let geometry = with_states(toplevel.wl_surface(), |states| {
        let mut guard = states.cached_state.get::<SurfaceCachedState>();
        guard
            .current()
            .geometry
            .clone()
    });

    if let Some(geometry) = geometry {
        jtoplevel.update_geometry(
            env,
            geometry.loc.x,
            geometry.loc.y,
            geometry.size.w,
            geometry.size.h,
        )?;
    } else {
        jtoplevel.default_geometry(env)?;
    }

    Ok(())
}

pub fn toplevel_from_java_nullable<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    jtoplevel: &WLCToplevel<'local>,
) -> Result<Option<ToplevelSurface>, BridgeError> {
    if jtoplevel.is_null() {
        return Ok(None);
    }

    let ptr = (jtoplevel.handle(env)? as usize) as *mut Weak<XdgToplevel>;
    if ptr.is_null() {
        return Err(BridgeError::ToplevelGone);
    }

    let weak = unsafe { &mut *ptr };
    let xdg_toplevel = weak.upgrade().map_err(|_| BridgeError::ToplevelGone)?;
    let toplevel = state.xdg_state.get_toplevel(&xdg_toplevel).unwrap();

    return Ok(Some(toplevel));
}

#[allow(unused)]
pub fn toplevel_from_java<'local>(
    env: &mut Env<'local>,
    state: &mut WLCState,
    jtoplevel: &WLCToplevel<'local>,
) -> Result<ToplevelSurface, BridgeError> {
    toplevel_from_java_nullable(env, state, jtoplevel)?
        .ok_or(BridgeError::ToplevelNull)
}

pub fn new_popup(_state: &mut WLCState, _popup: &PopupSurface) {
}

pub fn popup_destroyed(_state: &mut WLCState, _popup: &PopupSurface) {
}

pub fn toplevel_resize<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    jtoplevel: WLCToplevel<'local>,
    width: jint,
    height: jint,
    interactive: jboolean,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = toplevel_from_java(env, &mut instance.state, &jtoplevel)?;

    toplevel.with_pending_state(|state| {
        state.size = Some(Size::new(width, height));
        state.states.unset(xdg_toplevel::State::Maximized);
        state.states.unset(xdg_toplevel::State::Fullscreen);
        if interactive {
            state.states.set(xdg_toplevel::State::Resizing);
        } else {
            state.states.unset(xdg_toplevel::State::Resizing);
        }
    });
    jtoplevel.set_fullscreen(env, false)?;

    toplevel.send_pending_configure();

    Ok(())
}

pub fn toplevel_resize_ovr<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    jtoplevel: WLCToplevel<'local>,
    width: jint,
    height: jint,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = toplevel_from_java(env, &mut instance.state, &jtoplevel)?;

    toplevel.with_pending_state(|state| {
        state.size = Some(Size::new(width, height));
        state.states.unset(xdg_toplevel::State::Resizing);
    });

    toplevel.send_pending_configure();

    Ok(())
}

pub fn toplevel_maximize<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    jtoplevel: WLCToplevel<'local>,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = toplevel_from_java(env, &mut instance.state, &jtoplevel)?;

    toplevel.with_pending_state(|state| {
        if state.states.contains(xdg_toplevel::State::Fullscreen) {
            return;
        }
        let output = &instance.state.output;
        state.size = Some(output.bounds());
        state.states.set(xdg_toplevel::State::Maximized);
    });

    toplevel.send_configure();
    Ok(())
}

pub fn toplevel_fullscreen<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    jtoplevel: WLCToplevel<'local>,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = toplevel_from_java(env, &mut instance.state, &jtoplevel)?;

    toplevel.with_pending_state(|state| {
        let output = &instance.state.output;
        state.size = Some(output.size());
        state.states.set(xdg_toplevel::State::Fullscreen);
    });

    jtoplevel.set_fullscreen(env, true)?;

    toplevel.send_configure();
    Ok(())
}
