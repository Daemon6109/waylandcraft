use crate::{
    DmabufFeedbackData, WLCState, WaylandCraft,
    bridge::{java_types::*, sketchy::*},
};
use jni::{
    Env,
    objects::{
        JClass, JLongArray, JObject, JObjectArray, JPrimitiveArray, JString,
    },
    sys::{jint, jlong},
};
use rustix::{fd::AsRawFd, fs::makedev};
use smithay::backend::{
    allocator::{
        Buffer, Format, Fourcc, Modifier,
        dmabuf::{Dmabuf, WeakDmabuf},
    },
    drm::DrmNode,
};

pub fn dmabuf_feedback_from_java<'local>(
    env: &mut Env<'local>,
    jfeedback: JDmabufFeedbackData<'local>,
) -> Result<Option<DmabufFeedbackData>, BridgeError> {
    if jfeedback.is_null() {
        return Ok(None);
    }

    let device = jfeedback.drm_device(env)? as libc::dev_t;
    let formats = jfeedback.formats(env)?;
    let formats = JObjectArray::<JDmabufFormat>::cast_local(env, formats)?;
    let formats = formats_from_java(env, formats)?;
    Ok(Some(DmabufFeedbackData { device, formats }))
}

pub fn drm_device_by_path<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
) -> Result<jlong, BridgeError> {
    let path = path.try_to_string(env)?;
    let node = DrmNode::from_path(path)?;
    Ok(node.dev_id() as jlong)
}

pub fn drm_device_by_major_minor<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    major: jint,
    minor: jint,
) -> Result<jlong, BridgeError> {
    let id = makedev(major as u32, minor as u32);
    let node = DrmNode::from_dev_id(id)?;
    Ok(node.dev_id() as jlong)
}

fn formats_from_java<'local>(
    env: &mut Env<'local>,
    jformats: JObjectArray<'local, JDmabufFormat<'local>>,
) -> Result<Vec<Format>, BridgeError> {
    let mut formats: Vec<Format> = vec![];
    let len = jformats.len(env)?;
    for idx in 0..len {
        let jformat = jformats.get_element(env, idx)? as JDmabufFormat;
        let code = jformat.code(env)? as u32;
        let code = match Fourcc::try_from(code) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let modifier = jformat.modifier(env)? as u64;
        let modifier = Modifier::from(modifier);

        formats.push(Format { code, modifier });
    }

    Ok(formats)
}

pub fn check_import_dmabuf<'local>(
    env: &mut Env<'local>,
    this: WaylandCraftBridge<'local>,
    instance: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    let (dmabuf, notif) = match instance.state.pending_dmabuf_imports.pop() {
        Some(t) => t,
        None => return Ok(()),
    };

    let mut ref_box = Box::new(dmabuf.weak());
    let handle: *mut WeakDmabuf = &raw mut *ref_box;
    let handle = (handle as usize) as jlong;

    let jdmabuf = dmabuf_to_java(env, &dmabuf, handle)?;
    let success = this.import_dmabuf(env, jdmabuf)?;

    if !success {
        notif.failed();
        return Ok(());
    }

    match notif.successful::<WLCState>() {
        Ok(_) => {}
        Err(_) => return Ok(()),
    };

    instance.bridge.dmabufs.push(ref_box);
    Ok(())
}

fn dmabuf_to_java<'local>(
    env: &mut Env<'local>,
    dmabuf: &Dmabuf,
    dmabuf_handle: jlong,
) -> Result<JDmabuf<'local>, BridgeError> {
    let array = JObjectArray::<JDmabufPlane>::new(
        env,
        dmabuf.num_planes(),
        JDmabufPlane::null(),
    )?;

    let mut handles = dmabuf.handles();
    let mut offsets = dmabuf.offsets();
    let mut strides = dmabuf.strides();

    for idx in 0..dmabuf.num_planes() {
        let handle = handles.next().unwrap().as_raw_fd();
        let offset = offsets.next().unwrap();
        let stride = strides.next().unwrap();
        let plane =
            JDmabufPlane::new(env, handle, offset as jint, stride as jint)?;
        array.set_element(env, idx, plane)?;
    }

    let array = JObjectArray::<JObject>::cast_local(env, array)?;
    let jdmabuf = JDmabuf::new(
        env,
        dmabuf_handle,
        dmabuf.width() as jint,
        dmabuf.height() as jint,
        (dmabuf.format().code as u32) as jint,
        u64::from(dmabuf.format().modifier) as jlong,
        array,
    )?;

    Ok(jdmabuf)
}

pub fn dmabufs<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    instance.bridge.dmabufs.retain(|d| !d.is_gone());

    let handles = get_all_handles(&mut instance.bridge.dmabufs);
    let array = JLongArray::new(env, handles.len())?;
    array.set_region(env, 0, &handles)?;
    Ok(array)
}
