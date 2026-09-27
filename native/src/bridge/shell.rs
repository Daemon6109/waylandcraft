use crate::{
    WaylandCraft,
    bridge::{BridgeError, sketchy::*},
};
use jni::{
    Env,
    objects::{JClass, JIntArray, JLongArray, JPrimitiveArray, JString},
    sys::{jboolean, jint, jlong},
};
use smithay::{
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::Size,
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, SurfaceCachedState, ToplevelSurface,
            XdgToplevelSurfaceData,
        },
    },
};

pub fn toplevels<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    insert_all(
        &mut instance.bridge.toplevels,
        instance.state.xdg_state.toplevel_surfaces(),
    );

    instance.bridge.toplevels.retain(|t| t.alive());

    let toplevels = get_all_handles(&mut instance.bridge.toplevels);
    let array = JLongArray::new(env, toplevels.len())?;
    array.set_region(env, 0, &toplevels)?;
    Ok(array)
}

pub fn popups<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    insert_all(
        &mut instance.bridge.popups,
        instance.state.xdg_state.popup_surfaces(),
    );

    instance.bridge.popups.retain(|t| t.alive());
    let popups = get_all_handles(&mut instance.bridge.popups);

    let array = JLongArray::new(env, popups.len())?;
    array.set_region(env, 0, &popups)?;
    Ok(array)
}

pub enum RequestsVec {
    Minimize,
    Maximize,
    Unmaximize,
    Fullscreen,
    Unfullscreen,
}

pub fn clear_requests<'local>(
    env: &mut Env<'local>,
    instance: &mut WaylandCraft,
    vec: RequestsVec,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let vec = match vec {
        RequestsVec::Minimize => &mut instance.state.requests.minimize,
        RequestsVec::Maximize => &mut instance.state.requests.maximize,
        RequestsVec::Unmaximize => &mut instance.state.requests.unmaximize,
        RequestsVec::Fullscreen => &mut instance.state.requests.fullscreen,
        RequestsVec::Unfullscreen => &mut instance.state.requests.unfullscreen,
    };

    let handles: Vec<jlong> = vec
        .iter()
        .filter(|t| t.alive())
        .map(|t| insert_get_handle(&mut instance.bridge.toplevels, t))
        .collect();

    vec.clear();

    let array = JLongArray::new(env, handles.len())?;
    array.set_region(env, 0, &handles)?;
    Ok(array)
}

pub fn minimize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    clear_requests(env, instance, RequestsVec::Minimize)
}

pub fn maximize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    clear_requests(env, instance, RequestsVec::Maximize)
}

pub fn unmaximize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    clear_requests(env, instance, RequestsVec::Unmaximize)
}

pub fn fullscreen_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    clear_requests(env, instance, RequestsVec::Fullscreen)
}

pub fn unfullscreen_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    clear_requests(env, instance, RequestsVec::Unfullscreen)
}

pub fn move_request<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let serial = instance.state.requests.move_interactive.pop();

    let serial = match serial {
        Some(s) => s,
        None => return Ok(JPrimitiveArray::null()),
    };

    let serial = u32::from(serial) as jint;

    let array = JIntArray::new(env, 1)?;
    array.set_region(env, 0, &[serial])?;
    Ok(array)
}

pub fn resize_request<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let req = instance.state.requests.resize_interactive.pop();

    let (serial, edges) = match req {
        Some(r) => r,
        None => return Ok(JPrimitiveArray::null()),
    };

    let serial = u32::from(serial) as jint;
    let edges = u32::from(edges) as jint;

    let array = JIntArray::new(env, 2)?;
    array.set_region(env, 0, &[serial, edges])?;
    Ok(array)
}

pub fn toplevel_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<jlong, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelSurface")?;

    let surface = toplevel.wl_surface();

    Ok(insert_get_handle(&mut instance.bridge.surfaces, surface))
}

pub fn popup_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    popup_handle: jlong,
) -> Result<jlong, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let popup = jptr_to_popup!(popup_handle, "popupSurface")?;

    let surface = popup.wl_surface();

    Ok(insert_get_handle(&mut instance.bridge.surfaces, surface))
}

pub fn popup_parent<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    popup_handle: jlong,
) -> Result<jlong, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let popup = jptr_to_popup!(popup_handle, "popupParent")?;

    let parent_surface = match popup.get_parent_surface() {
        None => return Ok(0),
        Some(parent_surface) => parent_surface,
    };

    for toplevel in &instance.bridge.toplevels {
        if *toplevel.wl_surface() == parent_surface {
            return Ok(get_handle(&instance.bridge.toplevels, toplevel));
        }
    }

    for popup in &instance.bridge.popups {
        if *popup.wl_surface() == parent_surface {
            return Ok(get_handle(&instance.bridge.popups, popup));
        }
    }

    Ok(0)
}

pub fn popup_offset<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    popup_handle: jlong,
) -> Result<JPrimitiveArray<'local, jint>, BridgeError> {
    let popup = jptr_to_popup!(popup_handle, "popupOffset")?;

    let mut offset: [jint; 2] = [0, 0];

    popup.with_cached_state(|state| {
        let position = state.last_acked.map(|c| c.state.geometry.loc);

        if let Some(pos) = position {
            offset[0] = pos.x;
            offset[1] = pos.y;
        }
    });

    let array = JIntArray::new(env, 2)?;
    array.set_region(env, 0, &offset)?;
    Ok(array)
}

pub fn surface_xdg_geometry<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    surface_handle: jlong,
) -> Result<JPrimitiveArray<'local, jint>, BridgeError> {
    let surface = match jptr_to_ref(surface_handle) {
        Some(s) => s,
        None => return Ok(JIntArray::null()),
    };

    let geometry: Option<[jint; 4]> = with_states(surface, |states| {
        let mut guard = states.cached_state.get::<SurfaceCachedState>();
        guard
            .current()
            .geometry
            .map(|r| [r.loc.x, r.loc.y, r.size.w, r.size.h])
    });

    if let Some(geometry) = geometry {
        let array = JIntArray::new(env, 4)?;
        array.set_region(env, 0, &geometry)?;
        Ok(array)
    } else {
        Ok(JIntArray::null())
    }
}

pub fn toplevel_title<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
) -> Result<JString<'local>, BridgeError> {
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelTitle")?;

    let surface = toplevel.wl_surface();

    let title = with_states(surface, |states| {
        let attr_guard = states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .unwrap()
            .lock()
            .unwrap();

        attr_guard.title.clone()
    });

    if let Some(title) = title {
        Ok(env.new_string(title)?)
    } else {
        Ok(JString::null())
    }
}

pub fn toplevel_app_id<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
) -> Result<JString<'local>, BridgeError> {
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelAppId")?;

    let surface = toplevel.wl_surface();

    let app_id = with_states(surface, |states| {
        let attr_guard = states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .unwrap()
            .lock()
            .unwrap();

        attr_guard.app_id.clone()
    });

    if let Some(app_id) = app_id {
        Ok(env.new_string(app_id)?)
    } else {
        Ok(JString::null())
    }
}

pub fn toplevel_resize<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
    width: jint,
    height: jint,
    interactive: jboolean,
) -> Result<(), BridgeError> {
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelResize")?;

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

    toplevel.send_pending_configure();

    Ok(())
}

pub fn toplevel_resize_ovr<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
    width: jint,
    height: jint,
) -> Result<(), BridgeError> {
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelResizeOvr")?;

    toplevel.with_pending_state(|state| {
        state.size = Some(Size::new(width, height));
        state.states.unset(xdg_toplevel::State::Resizing);
    });

    toplevel.send_pending_configure();

    Ok(())
}

pub fn toplevel_maximize<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelMaximize")?;

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
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel = jptr_to_toplevel!(toplevel_handle, "toplevelFullscreen")?;

    toplevel.with_pending_state(|state| {
        let output = &instance.state.output;
        state.size = Some(output.size());
        state.states.set(xdg_toplevel::State::Fullscreen);
    });

    toplevel.send_configure();
    Ok(())
}

pub fn fullscreened<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let mut handles: Vec<jlong> = vec![];
    for toplevel in instance.state.xdg_state.toplevel_surfaces() {
        let fullscreen = toplevel.with_committed_state(|state| {
            state
                .map(|s| s.states.contains(xdg_toplevel::State::Fullscreen))
                .unwrap_or(false)
        });

        if !fullscreen {
            continue;
        }

        handles
            .push(insert_get_handle(&mut instance.bridge.toplevels, toplevel));
    }

    let array = JLongArray::new(env, handles.len())?;
    array.set_region(env, 0, &handles)?;
    Ok(array)
}

pub fn free_toplevel<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    remove_element(&mut instance.bridge.toplevels, toplevel_handle);

    Ok(())
}

pub fn free_popup<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    popup_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    remove_element(&mut instance.bridge.popups, popup_handle);

    Ok(())
}
