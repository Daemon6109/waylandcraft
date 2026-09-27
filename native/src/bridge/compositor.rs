use crate::{
    WaylandCraft,
    bridge::{java_types::*, sketchy::*},
    utils::get_time,
};
use jni::{
    Env,
    objects::JClass,
    sys::{jbyte, jint, jlong},
};
use smithay::{
    backend::allocator::Buffer,
    reexports::wayland_server::protocol::{
        wl_buffer::WlBuffer, wl_surface::WlSurface,
    },
    utils::Size,
    wayland::{
        compositor::{
            BufferAssignment, Damage, SubsurfaceCachedState, SurfaceAttributes,
            SurfaceData, TraversalAction, with_states,
            with_states as with_surface_data, with_surface_tree_upward,
        },
        dmabuf::get_dmabuf,
        shm::{self, with_buffer_contents},
        single_pixel_buffer::get_single_pixel_buffer,
        viewporter::{ViewportCachedState, ensure_viewport_valid},
    },
};
use std::ops::DerefMut;

pub fn send_frame<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    surface_handle: jlong,
) -> Result<(), BridgeError> {
    let surface = jptr_to_wl_surface!(surface_handle, "sendFrame")?;

    with_surface_data(surface, |data| {
        let mut attr_guard = data.cached_state.get::<SurfaceAttributes>();
        let attr = attr_guard.deref_mut().current();
        for c in attr.frame_callbacks.drain(..) {
            c.done(get_time());
        }
    });

    Ok(())
}

#[derive(PartialEq)]
enum BufferAttachResult {
    Success,
    TryAgain,
    Error,
    NotManaged,
}

fn try_attach_shm(
    _instance: &mut WaylandCraft,
    env: &mut Env,
    jsurface: &WLCSurface,
    buf: &WlBuffer,
    surf_data: &SurfaceData,
) -> BufferAttachResult {
    let r = with_buffer_contents(buf, |ptr, _len, metadata| {
        let width = metadata.width as jint;
        let height = metadata.height as jint;
        let format = (metadata.format as u32) as jint;
        let stride = metadata.stride as jint;
        ensure_viewport_valid(surf_data, Size::new(width, height));

        let ptr =
            unsafe { ptr.offset(metadata.offset as isize) }.addr() as jlong;

        jsurface
            .attach_shm_buffer(env, ptr, width, height, format, stride)
            .unwrap();
    });

    match r {
        Ok(_) => BufferAttachResult::Success,
        Err(shm::BufferAccessError::NotManaged) => {
            BufferAttachResult::NotManaged
        }
        Err(_) => BufferAttachResult::Error,
    }
}

fn try_attach_single_pixel(
    _instance: &mut WaylandCraft,
    env: &mut Env,
    jsurface: &WLCSurface,
    buf: &WlBuffer,
    surf_data: &SurfaceData,
) -> BufferAttachResult {
    let pix = match get_single_pixel_buffer(buf) {
        Ok(p) => p,
        Err(_) => {
            return BufferAttachResult::NotManaged;
        }
    };

    ensure_viewport_valid(surf_data, Size::new(1, 1));

    let [r, g, b, a] = pix.rgba8888();
    jsurface
        .attach_single_pixel_buffer(
            env, r as jbyte, g as jbyte, b as jbyte, a as jbyte,
        )
        .unwrap();

    BufferAttachResult::Success
}

fn try_attach_dmabuf(
    instance: &mut WaylandCraft,
    env: &mut Env,
    jsurface: &WLCSurface,
    buf: &WlBuffer,
    surf_data: &SurfaceData,
) -> BufferAttachResult {
    let dmabuf = match get_dmabuf(buf) {
        Ok(d) => d,
        Err(_) => return BufferAttachResult::NotManaged,
    };

    let width = dmabuf.width() as jint;
    let height = dmabuf.height() as jint;
    ensure_viewport_valid(surf_data, Size::new(width, height));

    let weak = dmabuf.weak();
    let handle = match get_handle_safe(&instance.bridge.dmabufs, &weak) {
        Some(h) => h,
        None => {
            // Client attempted to attach unknown dmabuf
            // This can happen when a new dmabuf has been created but hasn't
            // finished importing yet
            return BufferAttachResult::TryAgain;
        }
    };

    if jsurface.attach_dmabuf(env, handle).unwrap() {
        BufferAttachResult::Success
    } else {
        BufferAttachResult::Error
    }
}

// Proxy to call the try_attach_* family of functions
fn try_attach_buffer(
    instance: &mut WaylandCraft,
    env: &mut Env,
    jsurface: &WLCSurface,
    buf: &WlBuffer,
    surf_data: &SurfaceData,
) -> BufferAttachResult {
    type TryAttachFn = fn(
        instance: &mut WaylandCraft,
        env: &mut Env,
        jsurface: &WLCSurface,
        buf: &WlBuffer,
        surf_data: &SurfaceData,
    ) -> BufferAttachResult;

    let funcs: [TryAttachFn; 3] =
        [try_attach_shm, try_attach_single_pixel, try_attach_dmabuf];
    for func in funcs {
        let result = func(instance, env, jsurface, buf, surf_data);
        match result {
            BufferAttachResult::NotManaged => continue,
            a => return a,
        }
    }

    unreachable!("Buffer did not match any attachment mechanism!")
}

pub fn update_surface_data<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    jsurface: WLCSurface<'local>,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let handle = jsurface.handle(env)?;
    let surface = jptr_to_ref::<WlSurface>(handle).ok_or_else(|| {
        BridgeError::Null("updateSufaceData: surfaceHandle is null")
    })?;

    with_states(surface, |data| {
        let mut attr_guard = data.cached_state.get::<SurfaceAttributes>();
        let attr = attr_guard.deref_mut().current();

        let (maybe_buf, mut remove_buf) = if let Some(assign) = &attr.buffer {
            match assign {
                BufferAssignment::NewBuffer(b) => (Some(b), false),
                BufferAssignment::Removed => (None, true),
            }
        } else {
            (None, false)
        };

        if let Some(buf) = maybe_buf {
            let r = try_attach_buffer(instance, env, &jsurface, buf, data);
            if r == BufferAttachResult::Error {
                eprintln!("Buffer attach failed!");
                remove_buf = true;
            }

            // Done with buffer attachment
            // All buffers are immediately released because at this point they
            // are all already written to an independent GPU texture.
            // (including the dmabufs)
            if r != BufferAttachResult::TryAgain {
                buf.release();
                attr.buffer = None;
            }
        }

        if remove_buf {
            jsurface.remove_buffer(env).unwrap();
        }

        let mut vp_data_guard = data.cached_state.get::<ViewportCachedState>();
        let vp_data = vp_data_guard.deref_mut().current();

        if let Some(src) = vp_data.src {
            jsurface
                .set_viewport_src(
                    env, src.loc.x, src.loc.y, src.size.w, src.size.h,
                )
                .unwrap();
        }

        if let Some(dst) = vp_data.dst {
            jsurface.set_viewport_dst(env, dst.w, dst.h).unwrap();
        }

        jsurface.clear_damage(env).unwrap();
        for damage in &attr.damage {
            match damage {
                Damage::Surface(d) => {
                    jsurface
                        .add_surface_damage(
                            env, d.loc.x, d.loc.y, d.size.w, d.size.h,
                        )
                        .unwrap();
                }
                Damage::Buffer(d) => {
                    jsurface
                        .add_buffer_damage(
                            env, d.loc.x, d.loc.y, d.size.w, d.size.h,
                        )
                        .unwrap();
                }
            }
        }
        attr.damage.clear();
    });

    Ok(())
}

pub fn update_surface_tree<'local>(
    env: &mut Env<'local>,
    this: WaylandCraftBridge<'local>,
    instance: jlong,
    surface: WLCSurface<'local>,
) -> Result<WLCSurface<'local>, BridgeError> {
    let instance = jptr_to_instance!(instance)?;

    let handle = surface.handle(env)?;
    let surface = jptr_to_ref::<WlSurface>(handle).ok_or_else(|| {
        BridgeError::Null("updateSufaceTree: surface is not alive")
    })?;

    let mut last_child = WLCSurface::null();

    with_surface_tree_upward(
        surface,
        None,
        |surface, _data, _parent| {
            TraversalAction::DoChildren(Some(surface.clone()))
        },
        |surface, data, parent| {
            let handle =
                insert_get_handle(&mut instance.bridge.surfaces, surface);
            let surface = this.get_or_create_surface(env, handle).unwrap();

            // Set the WLCSurface parentHandle
            let parent_handle = if let Some(p) = parent {
                insert_get_handle(&mut instance.bridge.surfaces, p)
            } else {
                0
            };

            surface.set_parent_handle(env, parent_handle).unwrap();

            // Set last child to point to this current surface
            if !last_child.is_null() {
                last_child.set_next_child(env, &surface).unwrap();
            }

            // Set this surfaces nextChild to null
            surface.set_next_child(env, WLCSurface::null()).unwrap();

            // Set this surfaces prevChild to the last child
            surface.set_prev_child(env, &last_child).unwrap();

            // Mark this surface as visited
            surface.set_visited(env, true).unwrap();

            // Set subsurface location
            let (sx, sy) = if data.cached_state.has::<SubsurfaceCachedState>() {
                let mut subattr_guard =
                    data.cached_state.get::<SubsurfaceCachedState>();
                let subattr = subattr_guard.deref_mut().current();
                (subattr.location.x, subattr.location.y)
            } else {
                (0, 0)
            };

            surface.set_xoff(env, sx).unwrap();
            surface.set_yoff(env, sy).unwrap();

            last_child = surface;
        },
        |_surface, _data, _parent| true,
    );

    Ok(last_child)
}

pub fn free_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
) -> Result<(), BridgeError> {
    let instance = jptr_to_instance!(instance)?;
    remove_element(&mut instance.bridge.surfaces, surface_handle);

    Ok(())
}
