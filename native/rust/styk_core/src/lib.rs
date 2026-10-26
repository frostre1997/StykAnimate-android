use std::sync::Mutex;

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jlong;

mod renderer;
use renderer::Renderer;

static LOGGER: std::sync::Once = std::sync::Once::new();

#[no_mangle]
pub extern "system" fn Java_com_styk_animate_StykCore_nativeCreate(
    _env: JNIEnv, _class: JClass,
) -> jlong {
    LOGGER.call_once(|| {
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::Info),
        );
    });
    log::info!("nativeCreate called");
    let boxed = Box::new(Mutex::new(Renderer::new()));
    Box::into_raw(boxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_styk_animate_StykCore_nativeDestroy(
    _env: JNIEnv, _class: JClass, handle: jlong,
) {
    if handle != 0 {
        unsafe {
            drop(Box::from_raw(handle as *mut Mutex<Renderer>));
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_styk_animate_StykCore_nativeAttachSurface(
    env: *mut jni::sys::JNIEnv,
    _class: jni::sys::jclass,
    handle: jlong,
    surface: jni::sys::jobject,
) {
    let renderer = unsafe { &*(handle as *const Mutex<Renderer>) };
    let mut r = renderer.lock().unwrap();

    // from_surface wants raw jni-sys pointers: *mut JNIEnv and jobject
    let native_window = unsafe {
        ndk::native_window::NativeWindow::from_surface(env, surface)
            .expect("Failed to obtain ANativeWindow from Surface")
    };

    r.attach_surface(native_window);
}

#[no_mangle]
pub extern "system" fn Java_com_styk_animate_StykCore_nativeDetachSurface(
    _env: JNIEnv, _class: JClass, handle: jlong,
) {
    let renderer = unsafe { &*(handle as *const Mutex<Renderer>) };
    let mut r = renderer.lock().unwrap();
    r.detach_surface();
}

#[no_mangle]
pub extern "system" fn Java_com_styk_animate_StykCore_nativeRenderFrame(
    _env: JNIEnv, _class: JClass, handle: jlong, frame_time_nanos: jlong,
) {
    let renderer = unsafe { &*(handle as *const Mutex<Renderer>) };
    let mut r = renderer.lock().unwrap();
    r.render_frame(frame_time_nanos as f64 / 1_000_000_000.0);
}
