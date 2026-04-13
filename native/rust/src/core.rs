use image::{ImageBuffer as RgbaImageBuffer, Rgba};
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::ptr::{self, NonNull};
use std::slice;

pub const IMAGE_OK: i32 = 0;
pub const IMAGE_ERROR: i32 = 1;

#[repr(C)]
pub struct ImageBuffer {
    pub data: *mut u8,
    pub release_handle: *mut std::ffi::c_void,
    pub width: i32,
    pub height: i32,
    pub channels: i32,
    pub stride: i32,
}

#[repr(C)]
pub struct ImageResult {
    pub code: i32,
    pub buffer: ImageBuffer,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

struct BufferRelease {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

fn empty_buffer() -> ImageBuffer {
    ImageBuffer {
        data: ptr::null_mut(),
        release_handle: ptr::null_mut(),
        width: 0,
        height: 0,
        channels: 0,
        stride: 0,
    }
}

pub fn success(mut data: Vec<u8>, width: i32, height: i32, channels: i32) -> ImageResult {
    eprintln!("Using Rust image acceleration");

    let stride = width.saturating_mul(channels);
    let ptr = data.as_mut_ptr();
    let len = data.len();
    let capacity = data.capacity();
    let release_handle = Box::into_raw(Box::new(BufferRelease {
        ptr: NonNull::new(ptr).expect("vector pointer should not be null"),
        len,
        capacity,
    })) as *mut std::ffi::c_void;
    std::mem::forget(data);

    ImageResult {
        code: IMAGE_OK,
        buffer: ImageBuffer {
            data: ptr,
            release_handle,
            width,
            height,
            channels,
            stride,
        },
    }
}

pub fn error(message: impl Into<String>) -> ImageResult {
    let message = sanitize_message(message.into());
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(CString::new(message).expect("sanitized error message"));
    });

    ImageResult {
        code: IMAGE_ERROR,
        buffer: empty_buffer(),
    }
}

fn sanitize_message(message: String) -> String {
    if message.as_bytes().contains(&0) {
        message.replace('\0', " ")
    } else {
        message
    }
}

pub unsafe fn input_rgba_image<'a>(
    data: *const u8,
    width: i32,
    height: i32,
    channels: i32,
) -> Result<RgbaImageBuffer<Rgba<u8>, &'a [u8]>, String> {
    if data.is_null() {
        return Err("input buffer is null".to_string());
    }
    if width <= 0 || height <= 0 {
        return Err("image dimensions must be positive".to_string());
    }
    if channels != 4 {
        return Err(format!("expected 4 channels, got {channels}"));
    }

    let len = width as usize * height as usize * channels as usize;
    let bytes = unsafe { slice::from_raw_parts(data, len) };
    RgbaImageBuffer::from_raw(width as u32, height as u32, bytes)
        .ok_or_else(|| "failed to create RGBA image".to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn image_free_buffer(release_handle: *mut std::ffi::c_void) {
    if release_handle.is_null() {
        return;
    }

    let release = unsafe { Box::from_raw(release_handle as *mut BufferRelease) };
    unsafe {
        let _ = Vec::from_raw_parts(release.ptr.as_ptr(), release.len, release.capacity);
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn image_last_error_message() -> *const c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|message| message.as_ptr())
            .unwrap_or(CStr::from_bytes_with_nul(b"\0").unwrap().as_ptr())
    })
}
