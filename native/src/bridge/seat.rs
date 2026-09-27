use crate::{
    WaylandCraft,
    bridge::{java_types::*, sketchy::*},
};
use jni::{
    Env,
    objects::{JClass, JString},
    sys::{jboolean, jdouble, jint, jlong},
};
use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            Resource,
            protocol::{
                wl_keyboard::KeyState,
                wl_pointer::{Axis, ButtonState},
            },
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{SurfaceAttributes, with_states},
        shell::xdg::ToplevelSurface,
    },
};
use std::ops::DerefMut;

pub fn pointer_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.pointer_motion(x, y);

    Ok(())
}

pub fn pointer_motion_focus<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let surface = jptr_to_ref(surface_handle);
    instance.state.seat.pointer_motion_focus(surface, x, y);

    Ok(())
}

pub fn pointer_rel_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    dx: jdouble,
    dy: jdouble,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.pointer_relative_motion(dx, dy);

    Ok(())
}

pub fn maybe_pointer_lock<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
) -> Result<jboolean, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let Some(surface) = jptr_to_ref(surface_handle) else {
        return Ok(false);
    };

    Ok(instance.state.seat.pointer_lock(surface))
}

pub fn pointer_unlock<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.pointer_unlock();

    Ok(())
}

pub fn pointer_leave<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.pointer_motion_focus(None, 0.0, 0.0);

    Ok(())
}

pub fn pointer_button<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    button: jint,
    state: jint,
) -> Result<jint, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let state = match state {
        0 => ButtonState::Released,
        1 => ButtonState::Pressed,
        _ => return Err(BridgeError::UnknownPointerButton(state)),
    };

    Ok(instance.state.seat.pointer_button(button as u32, state) as jint)
}

pub fn pointer_axis<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    axis: jint,
    value: jdouble,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let axis = match axis {
        0 => Axis::VerticalScroll,
        1 => Axis::HorizontalScroll,
        _ => {
            return Err(BridgeError::UnknownScrollDirection(axis));
        }
    };

    instance.state.seat.pointer_axis(axis, value);

    Ok(())
}

pub fn cursor_shape<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<jint, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let shape = match instance.state.seat.cursor_shape {
        Some(shape) => shape as jint,
        None => -1,
    };

    Ok(shape)
}

pub fn keyboard_focus<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let toplevel: Option<&ToplevelSurface> = jptr_to_ref(surface_handle);

    let surface = toplevel.map(|t| t.wl_surface().clone());

    // Update the client gaining keyboard focus with the clipboard contents
    let client = surface.as_ref().and_then(|s| s.client());
    instance.state.data.update_clipboard_client(client);

    match surface {
        Some(s) => instance.state.seat.keyboard_focus(s),
        None => instance.state.seat.keyboard_unfocus(),
    };

    instance
        .state
        .xdg_state
        .toplevel_surfaces()
        .iter()
        .for_each(|t| {
            t.with_pending_state(|state| {
                state.states.unset(xdg_toplevel::State::Activated);
            });
        });

    if let Some(t) = toplevel {
        t.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        })
    }

    instance
        .state
        .xdg_state
        .toplevel_surfaces()
        .iter()
        .for_each(|t| {
            t.send_pending_configure();
        });

    Ok(())
}

pub fn keyboard_activate<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.activate_keyboard();

    Ok(())
}

pub fn keyboard_deactivate<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.deactivate_keyboard();

    Ok(())
}

pub fn keyboard_input<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    scancode: jint,
    action: jint,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let scancode = scancode as u32;
    let action = match action {
        0 => KeyState::Released,
        1 => KeyState::Pressed,
        _ => {
            return Err(BridgeError::UnknownKeyboardState(action));
        }
    };

    instance.state.seat.keyboard_key(scancode, action);

    Ok(())
}

pub fn keyboard_update<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    scancode: jint,
    pressed: jboolean,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance
        .state
        .seat
        .keyboard_update_xkb(scancode as u32, pressed);

    Ok(())
}

pub fn check_input_region<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    surface_handle: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<jboolean, BridgeError> {
    let Some(surface) = jptr_to_ref(surface_handle) else {
        return Ok(false);
    };

    let point: Point<f64, Logical> = Point::new(x, y);

    Ok(with_states(surface, |data| {
        let mut attr_guard = data.cached_state.get::<SurfaceAttributes>();
        let attr = attr_guard.deref_mut().current();
        if let Some(r) = &attr.input_region {
            r.contains(point.to_i32_floor())
        } else {
            true
        }
    }))
}

pub fn set_keymap_default<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.seat.change_keymap_to_default();

    Ok(())
}

pub fn export_keymap<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JString<'local>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let keymap_str = instance.state.seat.export_keymap();
    Ok(JString::new(env, keymap_str)?)
}

pub fn set_keymap_from_str<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    keymap: JString<'local>,
) -> Result<jboolean, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let keymap_str = keymap.try_to_string(env)?;
    Ok(instance.state.seat.change_keymap_from_str(keymap_str))
}
