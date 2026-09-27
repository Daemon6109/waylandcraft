use crate::{
    WaylandCraft,
    bridge::{dmabuf::dmabuf_feedback_from_java, java_types::*, sketchy::*},
    wlc_init,
};
use jni::{
    Env,
    objects::{JClass, JString},
    sys::jlong,
};
use smithay::{
    backend::allocator::dmabuf::WeakDmabuf,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::shell::xdg::{PopupSurface, ToplevelSurface},
};
use std::time::Duration;

mod compositor;
mod desktop;
mod dmabuf;
mod dnd;
mod java_types;
mod output;
mod seat;
mod shell;
mod sketchy;

#[allow(clippy::vec_box)]
pub struct BridgeState {
    /* Handle collections */
    toplevels: Vec<Box<ToplevelSurface>>,
    popups: Vec<Box<PopupSurface>>,
    surfaces: Vec<Box<WlSurface>>,
    dmabufs: Vec<Box<WeakDmabuf>>,
}

impl BridgeState {
    pub fn new() -> Self {
        BridgeState {
            toplevels: vec![],
            popups: vec![],
            surfaces: vec![],
            dmabufs: vec![],
        }
    }
}

fn init<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    dmabuf_feedback: JDmabufFeedbackData<'local>,
) -> Result<jlong, BridgeError> {
    let dmabuf_feedback = dmabuf_feedback_from_java(env, dmabuf_feedback)?;
    let instance = wlc_init(dmabuf_feedback).map_err(BridgeError::Init)?;
    let instance_box = Box::new(instance);
    let ptr = Box::into_raw(instance_box);

    Ok(ptr.addr() as jlong)
}

fn shutdown<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    // This function acquires the instance from a raw pointer again and
    // drops it. Goes without saying that there shouldn't be any further
    // calls into the bridge after this.

    let ptr = instance as *mut WaylandCraft;
    let _ = unsafe { Box::from_raw(ptr) };

    Ok(())
}

fn dispatch_clients<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance
        .event_loop
        .dispatch(Some(Duration::ZERO), &mut instance.state)
        .unwrap();

    Ok(())
}

fn flush_display<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.display_handle.flush_clients().unwrap();

    Ok(())
}

fn socket<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JString<'local>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let socket = instance
        .state
        .socket
        .to_str()
        .ok_or(BridgeError::OsStringToUtf8)?;

    Ok(JString::new(env, socket)?)
}

fn x11_display<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JString<'local>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    if let Some(ref s) = instance.state.satellite {
        Ok(JString::new(env, s.get_display())?)
    } else {
        Ok(JString::null())
    }
}
