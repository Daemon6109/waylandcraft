#![allow(clippy::too_many_arguments)]

use crate::bridge;
use jni::{bind_java_type, sys::jint};
use smithay::backend::drm::CreateDrmNodeError;
use thiserror::Error;

bind_java_type! {
    rust_type = pub WLCSurface,
    java_type = dev.evvie.waylandcraft.bridge.WLCSurface,

    fields {
        handle: jlong,
        visited: jboolean,
        next_child: WLCSurface,
        prev_child: WLCSurface,
        parent_handle: jlong,
        xoff: jint,
        yoff: jint,
    },

    methods {
        pub fn remove_buffer(),
        pub fn set_viewport_src(
            x: jdouble,
            y: jdouble,
            width: jdouble,
            height: jdouble
        ),
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
        fn get_or_create_surface(jlong) -> WLCSurface,
        fn import_dmabuf(JDmabuf) -> jboolean,
    },

    native_methods {
        static extern fn init {
            sig = (
                dmabuf_feedback: JDmabufFeedbackData,
            ) -> jlong,
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
        static extern fn x11_display {
            sig = (instance: jlong) -> JString,
            fn = bridge::x11_display,
        },
        static extern fn send_frame {
            sig = (surface_handle: jlong),
            fn = bridge::compositor::send_frame,
        },
        static extern fn update_surface_data {
            sig = (instance: jlong, surface: WLCSurface),
            fn = bridge::compositor::update_surface_data,
        },
        static extern fn toplevels {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::toplevels,
        },
        static extern fn toplevel_surface {
            sig = (instance: jlong, toplevel_handle: jlong) -> jlong,
            fn = bridge::shell::toplevel_surface,
        },
        static extern fn toplevel_title {
            sig = (toplevel_handle: jlong) -> JString,
            fn = bridge::shell::toplevel_title,
        },
        static extern fn toplevel_app_id {
            sig = (toplevel_handle: jlong) -> JString,
            name = "toplevelAppID",
            fn = bridge::shell::toplevel_app_id,
        },
        static extern fn toplevel_resize {
            sig = (
                toplevel_handle: jlong,
                width: jint,
                height: jint,
                interactive: jboolean
            ),
            fn = bridge::shell::toplevel_resize,
        },
        static extern fn toplevel_resize_ovr {
            sig = (toplevel_handle: jlong, width: jint, height: jint),
            fn = bridge::shell::toplevel_resize_ovr,
        },
        static extern fn minimize_req {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::minimize_req,
        },
        static extern fn maximize_req {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::maximize_req,
        },
        static extern fn unmaximize_req {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::unmaximize_req,
        },
        static extern fn fullscreen_req {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::fullscreen_req,
        },
        static extern fn unfullscreen_req {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::unfullscreen_req,
        },
        static extern fn move_request {
            sig = (instance: jlong) -> jint[],
            fn = bridge::shell::move_request,
        },
        static extern fn resize_request {
            sig = (instance: jlong) -> jint[],
            fn = bridge::shell::resize_request,
        },
        static extern fn fullscreened {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::fullscreened,
        },
        static extern fn toplevel_maximize {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = bridge::shell::toplevel_maximize,
        },
        static extern fn toplevel_fullscreen {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = bridge::shell::toplevel_fullscreen,
        },
        static extern fn popups {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::shell::popups,
        },
        static extern fn popup_surface {
            sig = (instance: jlong, popup_handle: jlong) -> jlong,
            fn = bridge::shell::popup_surface,
        },
        static extern fn popup_parent {
            sig = (instance: jlong, popup_handle: jlong) -> jlong,
            fn = bridge::shell::popup_parent,
        },
        static extern fn popup_offset {
            sig = (popup_handle: jlong) -> jint[],
            fn = bridge::shell::popup_offset,
        },
        static extern fn surface_xdg_geometry {
            sig = (surface_handle: jlong) -> jint[],
            name = "surfaceXDGGeometry",
            fn = bridge::shell::surface_xdg_geometry,
        },
        static extern fn dmabufs {
            sig = (instance: jlong) -> jlong[],
            fn = bridge::dmabuf::dmabufs
        },
        extern fn check_import_dmabuf {
            sig = (instance: jlong),
            fn = bridge::dmabuf::check_import_dmabuf,
        },
        extern fn update_surface_tree {
            sig = (instance: jlong, surface: WLCSurface) -> WLCSurface,
            fn = bridge::compositor::update_surface_tree,
        },
        static extern fn check_input_region {
            sig = (surface_handle: jlong, x: jdouble, y: jdouble) -> jboolean,
            fn = bridge::seat::check_input_region,
        },
        static extern fn pointer_motion {
            sig = (instance: jlong, x: jdouble, y: jdouble),
            fn = bridge::seat::pointer_motion,
        },
        static extern fn pointer_motion_focus {
            sig = (
                instance: jlong,
                surface_handle: jlong,
                x: jdouble,
                y: jdouble
            ),
            fn = bridge::seat::pointer_motion_focus,
        },
        static extern fn pointer_rel_motion {
            sig = (instance: jlong, dx: jdouble, dy: jdouble),
            fn = bridge::seat::pointer_rel_motion,
        },
        static extern fn maybe_pointer_lock {
            sig = (instance: jlong, surface_handle: jlong) -> jboolean,
            fn = bridge::seat::maybe_pointer_lock,
        },
        static extern fn pointer_unlock {
            sig = (instance: jlong),
            fn = bridge::seat::pointer_unlock,
        },
        static extern fn pointer_leave {
            sig = (instance: jlong),
            fn = bridge::seat::pointer_leave,
        },
        static extern fn pointer_button {
            sig = (instance: jlong, button: jint, state: jint) -> jint,
            fn = bridge::seat::pointer_button,
        },
        static extern fn pointer_axis {
            sig = (instance: jlong, axis: jint, value: jdouble),
            fn = bridge::seat::pointer_axis,
        },
        static extern fn cursor_shape {
            sig = (instance: jlong) -> jint,
            fn = bridge::seat::cursor_shape,
        },
        static extern fn keyboard_focus {
            sig = (instance: jlong, surface_handle: jlong),
            fn = bridge::seat::keyboard_focus,
        },
        static extern fn keyboard_activate {
            sig = (instance: jlong),
            fn = bridge::seat::keyboard_activate,
        },
        static extern fn keyboard_deactivate {
            sig = (instance: jlong),
            fn = bridge::seat::keyboard_deactivate,
        },
        static extern fn keyboard_input {
            sig = (instance: jlong, scancode: jint, action: jint),
            fn = bridge::seat::keyboard_input,
        },
        static extern fn keyboard_update {
            sig = (instance: jlong, scancode: jint, pressed: jboolean),
            fn = bridge::seat::keyboard_update,
        },
        static extern fn output_size {
            sig = (instance: jlong) -> jint[],
            fn = bridge::output::output_size,
        },
        static extern fn output_bounds {
            sig = (instance: jlong) -> jint[],
            fn = bridge::output::output_bounds,
        },
        static extern fn output_resize {
            sig = (instance: jlong, width: jint, height: jint),
            fn = bridge::output::output_resize,
        },
        static extern fn output_set_bounds {
            sig = (instance: jlong, width: jint, height: jint),
            fn = bridge::output::output_set_bounds,
        },
        static extern fn free_surface {
            sig = (instance: jlong, surface_handle: jlong),
            fn = bridge::compositor::free_surface,
        },
        static extern fn free_toplevel {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = bridge::shell::free_toplevel,
        },
        static extern fn free_popup {
            sig = (instance: jlong, popup_handle: jlong),
            fn = bridge::shell::free_popup,
        },
        static extern fn load_desktop_entry {
            sig = (instance: jlong, path: JString) -> JRawDesktopEntry,
            fn = bridge::desktop::load_desktop_entry,
        },
        static extern fn load_desktop_entries {
            sig = (instance: jlong) -> JRawDesktopEntry[],
            fn = bridge::desktop::load_desktop_entries,
        },
        static extern fn render_svg {
            sig = (
                path: JString,
                width: jint,
                height: jint,
                buffer_ptr: jlong
            ) -> jboolean,
            name = "renderSVG",
            fn = bridge::desktop::render_svg,
        },
        static extern fn exec_app {
            sig = (instance: jlong, app_id: JString) -> jboolean,
            fn = bridge::desktop::exec_app,
        },
        static extern fn set_preferred_terminal {
            sig = (instance: jlong, cmd: JString),
            fn = bridge::desktop::set_preferred_terminal,
        },
        static extern fn set_keymap_default {
            sig = (instance: jlong),
            fn = bridge::seat::set_keymap_default,
        },
        static extern fn export_keymap {
            sig = (instance: jlong) -> JString,
            fn = bridge::seat::export_keymap,
        },
        static extern fn set_keymap_from_str {
            sig = (instance: jlong, keymap: JString) -> jboolean,
            fn = bridge::seat::set_keymap_from_str,
        },
        static extern fn check_dnd_request {
            sig = (instance: jlong) -> jint[],
            fn = bridge::dnd::check_dnd_request,
        },
        static extern fn check_dnd_active {
            sig = (instance: jlong) -> jboolean,
            fn = bridge::dnd::check_dnd_active,
        },
        static extern fn dnd_cancel {
            sig = (instance: jlong),
            fn = bridge::dnd::dnd_cancel,
        },
        static extern fn dnd_drop {
            sig = (instance: jlong),
            fn = bridge::dnd::dnd_drop,
        },
        static extern fn dnd_motion {
            sig = (
                instance: jlong,
                surface_handle: jlong,
                x: jdouble,
                y: jdouble
            ),
            fn = bridge::dnd::dnd_motion,
        },
        static extern fn dnd_icon {
            sig = (instance: jlong) -> jlong,
            fn = bridge::dnd::dnd_icon,
        },
        static extern fn drm_device_by_path {
            sig = (path: JString) -> jlong,
            fn = bridge::dmabuf::drm_device_by_path,
        },
        static extern fn drm_device_by_major_minor {
            sig = (major: jint, minor: jint) -> jlong,
            fn = bridge::dmabuf::drm_device_by_major_minor,
        },
    },
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error(transparent)]
    JniError(#[from] jni::errors::Error),
    #[error(transparent)]
    Init(Box<dyn std::error::Error>),
    #[error("{0}")]
    Null(&'static str),
    #[error("Received null WLC instance")]
    NullInstancePtr,
    #[error("Null wayland surface handle given. Function: {0}")]
    NullSurfacePtr(&'static str),
    #[error("Null toplevel surface handle given. Function: {0}")]
    NullToplevelPtr(&'static str),
    #[error("Null popup surface handle given. Function: {0}")]
    NullPopupPtr(&'static str),
    #[error("Error converting OS string, was not UTF8")]
    OsStringToUtf8,
    #[error("Unknown pointer button {0} received")]
    UnknownPointerButton(jint),
    #[error("Unknown scroll direction {0} received")]
    UnknownScrollDirection(jint),
    #[error("Unknown keyboard state {0} received")]
    UnknownKeyboardState(jint),
    #[error("Width cannot be below 1")]
    NonPositiveWidth,
    #[error("Height cannot be below 1")]
    NonPositiveHeight,
    #[error(transparent)]
    DrmNodeError(#[from] CreateDrmNodeError),
}
