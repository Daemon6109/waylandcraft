use crate::{
    WaylandCraft,
    bridge::{java_types::*, sketchy::*},
};
use jni::{
    Env,
    objects::{JClass, JIntArray, JPrimitiveArray},
    sys::{jboolean, jdouble, jint, jlong},
};

pub fn check_dnd_request<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let serial = match instance.state.data.check_dnd_request() {
        Some(r) => r as jint,
        None => return Ok(JIntArray::null()),
    };

    let array = JIntArray::new(env, 1)?;
    array.set_region(env, 0, &[serial])?;
    Ok(array)
}

pub fn check_dnd_active<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<jboolean, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    Ok(instance.state.data.dnd.is_some())
}

pub fn dnd_cancel<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.data.dnd_cancel();

    Ok(())
}

pub fn dnd_drop<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.state.data.dnd_drop();

    Ok(())
}

pub fn dnd_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let surface = jptr_to_ref(surface_handle);
    instance.state.data.dnd_motion(surface, x, y);

    Ok(())
}

pub fn dnd_icon<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<jlong, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let Some(dnd) = &instance.state.data.dnd else {
        return Ok(0);
    };

    match dnd.icon.as_ref() {
        Some(icon) => {
            Ok(insert_get_handle(&mut instance.bridge.surfaces, icon))
        }
        None => Ok(0),
    }
}
