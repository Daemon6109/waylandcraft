#![allow(clippy::too_many_arguments)]

use crate::bridge;
use jni::{bind_java_type, sys::jint};
use smithay::backend::drm::CreateDrmNodeError;
use thiserror::Error;

bind_java_type! {
    rust_type = pub WLCSurface,
    java_type = dev.evvie.waylandcraft.bridge.WLCSurface,

    constructors {
        fn new(handle: jlong),
    },

    fields {
        handle: jlong,
        parent: WLCSurface,
        children: WLCSurface[],
        surface_draw_tree: WLCSurface[],
        surface_input_tree: WLCSurface[],
        xoff: jint,
        yoff: jint,
        x_subpos: jint,
        y_subpos: jint,
    },

    methods {
        pub fn remove_buffer(),
        pub fn set_viewport_src(
            x: jdouble,
            y: jdouble,
            width: jdouble,
            height: jdouble
        ),
        pub fn unset_viewport_src(),
        pub fn set_viewport_dst(width: jint, height: jint),
        pub fn attach_shm_buffer(
            ptr: jlong,
            width: jint,
            height: jint,
            format: jint,
            stride: jint
        ),
        pub fn attach_single_pixel_buffer(
            red: jbyte,
            green: jbyte,
            blue: jbyte,
            alpha: jbyte
        ),
        pub fn attach_dmabuf(handle: jlong) -> jboolean,
        pub fn clear_damage(),
        pub fn add_buffer_damage(x: jint, y: jint, width: jint, height: jint),
        pub fn add_surface_damage(x: jint, y: jint, width: jint, height: jint),
        pub fn calculate_subpos(),
        pub fn commit(),
    },

    native_methods {
        extern fn send_frame {
            sig = (),
            fn = bridge::compositor::send_frame,
        },
    },
}

bind_java_type! {
    rust_type = pub JDmabufFormat,
    java_type = dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat,

    constructors {
        fn new(
            code: jint,
            modifier: jlong
        )
    },

    fields {
        code: jint,
        modifier: jlong
    },
}

bind_java_type! {
    rust_type = pub JDmabufPlane,
    java_type = dev.evvie.waylandcraft.bridge.dmabuf.DmabufPlane,

    constructors {
        fn new(
            fd: jint,
            offset: jint,
            stride: jint
        )
    },
}

bind_java_type! {
    rust_type = pub JDmabuf,
    java_type = dev.evvie.waylandcraft.bridge.dmabuf.Dmabuf,

    constructors {
        fn new(
            handle: jlong,
            width: jint,
            height: jint,
            format: jint,
            modifier: jlong,
            planes: dev.evvie.waylandcraft.bridge.dmabuf.DmabufPlane[]
        )
    },
}

bind_java_type! {
    rust_type = pub JDmabufFeedbackData,
    java_type = dev.evvie.waylandcraft.bridge.dmabuf.DmabufFeedbackData,

    constructors {
        fn new(
            drm_device: jlong,
            formats: dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat[],
        )
    },

    fields {
        drm_device: jlong,
        formats: dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat[],
    },
}

bind_java_type! {
    rust_type = pub JRawDesktopEntry,
    java_type = dev.evvie.waylandcraft.desktop.RawDesktopEntry,

    constructors {
        fn new(
            app_id: JString,
            name: JString,
            generic_name: JString,
            exec: JString,
            exec_terminal: jboolean,
            comment: JString,
            keywords: JString[],
            categories: JString[],
            visible: jboolean,
            icon_path: JString
        )
    },
}

bind_java_type! {
    rust_type = pub WaylandCraftBridge,
    java_type = dev.evvie.waylandcraft.bridge.WaylandCraftBridge,

    type_map {
        WLCSurface => dev.evvie.waylandcraft.bridge.WLCSurface,
        JRawDesktopEntry => dev.evvie.waylandcraft.desktop.RawDesktopEntry,
        JDmabufFormat => dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat,
        JDmabufPlane => dev.evvie.waylandcraft.bridge.dmabuf.DmabufPlane,
        JDmabuf => dev.evvie.waylandcraft.bridge.dmabuf.Dmabuf,
        JDmabufFeedbackData =>
            dev.evvie.waylandcraft.bridge.dmabuf.DmabufFeedbackData,
    },

    methods {
        fn add_surface(WLCSurface),
        fn delete_surface(WLCSurface),
    },

    constructors {
        fn new(jlong),
    },

    native_methods {
        static extern fn init {
            sig = () -> WaylandCraftBridge,
            fn = bridge::init,
        },
        static extern fn shutdown {
            sig = (instance: jlong),
            fn = bridge::shutdown,
        },
        static extern fn dispatch_clients {
            sig = (instance: jlong),
            fn = bridge::dispatch_clients,
        },
        static extern fn flush_display {
            sig = (instance: jlong),
            fn = bridge::flush_display
        },
        static extern fn socket {
            sig = (instance: jlong) -> JString,
            fn = bridge::socket,
        },
    },
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error(transparent)]
    JniError(#[from] jni::errors::Error),
    #[error(transparent)]
    Init(Box<dyn std::error::Error>),
    #[error("Received null instance handle")]
    NullInstancePtr,
    #[error("Error converting OS string to UTF-8")]
    OsStringToUtf8,
    #[error("Surface is already gone")]
    SurfaceGone,
    #[error("Surface is null")]
    SurfaceNull,
}
