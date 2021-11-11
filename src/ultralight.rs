use std::ffi::CString;
use std::sync::{mpsc::*, Arc, Mutex};
use std::thread::{self, JoinHandle};

use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};
use ul_sys::*;

pub struct Ultralight {
    sender: Sender<ChannelType>,
    _join_handle: JoinHandle<()>,
}

struct UltralightThreadResources {
    config: ULConfig,
    renderer: ULRenderer,
    view: ULView,
}

type ChannelType = Box<dyn Fn(&UltralightThreadResources) + Send>;

impl Ultralight {
    pub fn new(width: u32, height: u32) -> Self {
        let log_path = CString::new("./ultralight.log").unwrap();
        let base_path = CString::new("./").unwrap();
        let res_path = CString::new("./resources/").unwrap();

        let (tx, rx) = channel::<ChannelType>();
        let join_handle = thread::spawn(move || {
            let res = unsafe {
                let config = ulCreateConfig();
                let c_log_path = ulCreateString(log_path.as_ptr());
                ulEnableDefaultLogger(c_log_path);
                ulEnablePlatformFontLoader();
                let c_base_path = ulCreateString(base_path.as_ptr());
                ulEnablePlatformFileSystem(c_base_path);
                let c_res_path = ulCreateString(res_path.as_ptr());
                ulConfigSetResourcePath(config, c_res_path);

                let renderer = ulCreateRenderer(config);
                let view = ulCreateView(renderer, width, height, true, std::ptr::null_mut(), true);

                ulDestroyString(c_log_path);
                ulDestroyString(c_base_path);
                ulDestroyString(c_res_path);

                UltralightThreadResources {
                    config,
                    renderer,
                    view,
                }
            };

            loop {
                if let Ok(task) = rx.recv() {
                    task(&res);
                }
            }
        });

        Self {
            sender: tx,
            _join_handle: join_handle,
        }
    }

    pub fn load_html(&self, html: &str) {
        let html = CString::new(html).unwrap();
        self.sender
            .send(Box::new(move |res| unsafe {
                let c_html = ulCreateString(html.as_ptr());
                ulViewLoadHTML(res.view, c_html);
                ulViewFocus(res.view);
                ulDestroyString(c_html);
            }))
            .unwrap();
    }

    pub fn update(&self) {
        self.sender
            .send(Box::new(|res| unsafe {
                ulUpdate(res.renderer);
                ulRender(res.renderer);
            }))
            .unwrap();
    }

    pub fn fire_mouse_motion_event(&self, position: Vec2) {
        self.sender
            .send(Box::new(move |res| unsafe {
                let c_motion = ulCreateMouseEvent(
                    ULMouseEventType_kMouseEventType_MouseMoved,
                    position.x as i32,
                    position.y as i32,
                    0,
                );
                ulViewFireMouseEvent(res.view, c_motion);
                ulDestroyMouseEvent(c_motion);
            }))
            .unwrap();
    }

    pub fn fire_mouse_button_event(
        &self,
        position: Vec2,
        button: MouseButton,
        state: ElementState,
    ) {
        self.sender
            .send(Box::new(move |res| unsafe {
                let c_motion = ulCreateMouseEvent(
                    match state {
                        ElementState::Pressed => ULMouseEventType_kMouseEventType_MouseDown,
                        ElementState::Released => ULMouseEventType_kMouseEventType_MouseUp,
                    },
                    position.x as i32,
                    position.y as i32,
                    match button {
                        MouseButton::Left => ULMouseButton_kMouseButton_Left,
                        MouseButton::Right => ULMouseButton_kMouseButton_Right,
                        MouseButton::Middle => ULMouseButton_kMouseButton_Middle,
                        _ => 0,
                    },
                );
                ulViewFireMouseEvent(res.view, c_motion);
                ulDestroyMouseEvent(c_motion);
            }))
            .unwrap();
    }

    pub fn execute_javascript(&self, code: &str) {
        let code = CString::new(code).unwrap();
        self.sender
            .send(Box::new(move |res| unsafe {
                let c_code = ulCreateString(code.as_ptr());
                ulViewEvaluateScript(res.view, c_code, std::ptr::null_mut());
                ulDestroyString(c_code);
            }))
            .unwrap();
    }

    pub fn receive_texture_buffer(&self) -> Arc<Mutex<Option<Texture>>> {
        let output = Arc::new(Mutex::new(None));
        let output_clone = output.clone();

        self.sender
            .send(Box::new(move |res| unsafe {
                let surface = ulViewGetSurface(res.view);
                let bitmap = ulBitmapSurfaceGetBitmap(surface);
                let pixels = ulBitmapLockPixels(bitmap);
                let height = ulBitmapGetHeight(bitmap);
                let width = ulBitmapGetWidth(bitmap);
                let stride = ulBitmapGetRowBytes(bitmap);

                let buffer =
                    std::slice::from_raw_parts(pixels as *mut u8, (stride * height) as usize);

                let texture = Texture::new(
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    buffer.to_vec(),
                    TextureFormat::Bgra8Unorm,
                );
                let mut output = output_clone.lock().unwrap();
                *output = Some(texture);

                ulBitmapUnlockPixels(bitmap);
            }))
            .unwrap();

        output
    }
}

impl Drop for UltralightThreadResources {
    fn drop(&mut self) {
        unsafe {
            ulDestroyView(self.view);
            ulDestroyRenderer(self.renderer);
            ulDestroyConfig(self.config);
        }
    }
}
