#![no_main]
#![no_std]

// Required for panic handler
extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use flipperzero_rt::{entry, manifest};
use sys::{
    Align_AlignCenter, GuiButtonType_GuiButtonTypeCenter, GuiButtonType_GuiButtonTypeLeft,
    GuiButtonType_GuiButtonTypeRight,
};

use core::ffi::{c_char, c_void, CStr};
use core::mem::{self, MaybeUninit};
use core::ptr;

use alloc::boxed::Box;
use core::ptr::NonNull;
use flipperzero::furi::string::FuriString;
use flipperzero_sys as sys;
use flipperzero_sys::furi::UnsafeRecord;

const FULLSCREEN: sys::GuiLayer = sys::GuiLayer_GuiLayerFullscreen;

enum AppView {
    Widget = 0,
    TextInput = 1,
    ImabeView = 2,
}

struct App {
    name: [c_char; 16],
    view_dispatcher: NonNull<sys::ViewDispatcher>,
    widget: NonNull<sys::Widget>,
    text_input: NonNull<sys::TextInput>,
}

impl App {
    pub fn new() -> Box<Self> {
        Box::new(App {
            name: Default::default(),
            view_dispatcher: unsafe { NonNull::new_unchecked(sys::view_dispatcher_alloc()) },
            widget: unsafe { NonNull::new_unchecked(sys::widget_alloc()) },
            text_input: unsafe { NonNull::new_unchecked(sys::text_input_alloc()) },
        })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            sys::view_dispatcher_free(self.view_dispatcher.as_ptr());
            sys::widget_free(self.widget.as_ptr());
            sys::text_input_free(self.text_input.as_ptr());
        }
    }
}

pub unsafe extern "C" fn text_input_callback(context: *mut c_void) {
    let app = context as *mut App;
    let mut message = FuriString::from("Hello ");
    message.push_c_str(CStr::from_ptr((*app).name.as_ptr()));
    sys::widget_add_string_element(
        (*app).widget.as_ptr(),
        128 / 2,
        64 / 2,
        sys::Align_AlignCenter,
        sys::Align_AlignCenter,
        sys::Font_FontPrimary,
        message.as_c_ptr(),
    );
    sys::widget_add_string_element(
        (*app).widget.as_ptr(),
        10,
        10,
        sys::Align_AlignCenter,
        sys::Align_AlignCenter,
        sys::Font_FontPrimary,
        FuriString::from("custom shit ").as_c_ptr(),
    );

    sys::widget_add_text_box_element(
        (*app).widget.as_ptr(),
        0,
        0,
        40,
        10,
        Align_AlignCenter,
        Align_AlignCenter,
        FuriString::from("Textovi").as_c_ptr(),
        true,
    );
    sys::widget_add_frame_element((*app).widget.as_ptr(), 15, 15, 48, 32, 10);
    sys::widget_add_icon_element(
        (*app).widget.as_ptr(),
        20,
        40,
        &TARGET_ICON as *const Icon as *const c_void as *const sys::Icon,
    );
    sys::widget_add_button_element(
        (*app).widget.as_ptr(),
        GuiButtonType_GuiButtonTypeCenter,
        FuriString::from("Press Me").as_c_ptr(),
        Some(button_callback),
        context,
    );
    sys::widget_add_button_element(
        (*app).widget.as_ptr(),
        GuiButtonType_GuiButtonTypeLeft,
        FuriString::from("Yes").as_c_ptr(),
        Some(button_callback),
        context,
    );
    sys::widget_add_button_element(
        (*app).widget.as_ptr(),
        GuiButtonType_GuiButtonTypeRight,
        FuriString::from("No").as_c_ptr(),
        Some(button_callback),
        context,
    );
    sys::view_dispatcher_switch_to_view((*app).view_dispatcher.as_ptr(), AppView::Widget as u32);
}

extern "C" fn button_callback(_button_type: u8, _button_action: u8, context: *mut c_void) {
    // Implementation remains similar
    let app = unsafe { &mut *(context as *mut App) };
    // Handle the button press event
}

pub unsafe extern "C" fn navigation_event_callback(context: *mut c_void) -> bool {
    let view_dispatcher = context as *mut sys::ViewDispatcher;
    sys::view_dispatcher_stop(view_dispatcher);
    sys::view_dispatcher_remove_view(view_dispatcher, AppView::Widget as u32);
    sys::view_dispatcher_remove_view(view_dispatcher, AppView::TextInput as u32);
    true
}

// Define the FAP Manifest for this application
manifest!(
    name = "Helldivers2",
    app_version = 1,
    has_icon = true,
    // See https://github.com/flipperzero-rs/flipperzero/blob/v0.7.2/docs/icons.md for icon format
    icon = "rustacean-10x10.icon",
);

// Define the entry function
entry!(main);

pub unsafe extern "C" fn draw_callback(canvas: *mut sys::Canvas, _context: *mut c_void) {
    unsafe {
        sys::canvas_draw_str(canvas, 30, 30, c"Hello Andrija".as_ptr());
    }
}

static mut TARGET_ICON: Icon = Icon {
    width: 48,
    height: 32,
    frame_count: 1,
    frame_rate: 0,
    frames: unsafe { TARGET_FRAMES.as_ptr() },
};

static mut TARGET_FRAMES: [*const u8; 1] = [include_bytes!("icons/rustacean-48x32.icon").as_ptr()];
static mut IMAGE_POSITION: ImagePosition = ImagePosition { x: 0, y: 0 };

#[repr(C)]
struct ImagePosition {
    pub x: u8,
    pub y: u8,
}

#[repr(C)]
struct Icon {
    width: u8,
    height: u8,
    frame_count: u8,
    frame_rate: u8,
    frames: *const *const u8,
}

extern "C" fn app_draw_callback(canvas: *mut sys::Canvas, _ctx: *mut c_void) {
    unsafe {
        sys::canvas_clear(canvas);
        sys::canvas_draw_str(canvas, 30, 30, c"Hello Andrija".as_ptr());
        sys::canvas_draw_line(canvas, 0, 0, 50, 50);
        sys::canvas_draw_icon(
            canvas,
            IMAGE_POSITION.x % 128,
            IMAGE_POSITION.y % 128,
            &TARGET_ICON as *const Icon as *const c_void as *const sys::Icon,
        );
    }
}

extern "C" fn app_input_callback(input_event: *mut sys::InputEvent, ctx: *mut c_void) {
    unsafe {
        let event_queue = ctx as *mut sys::FuriMessageQueue;
        sys::furi_message_queue_put(event_queue, input_event as *mut c_void, 0);
    }
}

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    let mut app = App::new();

    unsafe {
        sys::view_dispatcher_enable_queue(app.view_dispatcher.as_ptr());
        sys::view_dispatcher_set_event_callback_context(
            app.view_dispatcher.as_ptr(),
            app.view_dispatcher.as_ptr() as *mut c_void,
        );
        sys::view_dispatcher_set_navigation_event_callback(
            app.view_dispatcher.as_ptr(),
            Some(navigation_event_callback),
        );
        sys::view_dispatcher_add_view(
            app.view_dispatcher.as_ptr(),
            AppView::Widget as u32,
            sys::widget_get_view(app.widget.as_ptr()),
        );
        sys::view_dispatcher_add_view(
            app.view_dispatcher.as_ptr(),
            AppView::TextInput as u32,
            sys::text_input_get_view(app.text_input.as_ptr()),
        );
    }

    unsafe {
        let gui = UnsafeRecord::open(c"gui".as_ptr());
        sys::view_dispatcher_attach_to_gui(
            app.view_dispatcher.as_ptr(),
            gui.as_ptr(),
            sys::ViewDispatcherType_ViewDispatcherTypeFullscreen,
        );

        sys::text_input_reset(app.text_input.as_ptr());
        sys::text_input_set_header_text(app.text_input.as_ptr(), c"Enter your name".as_ptr());

        sys::text_input_set_result_callback(
            app.text_input.as_ptr(),
            Some(text_input_callback),
            &*app as *const App as *mut c_void,
            app.name.as_mut_ptr(),
            app.name.len(),
            true,
        );

        sys::view_dispatcher_switch_to_view(
            app.view_dispatcher.as_ptr(),
            AppView::TextInput as u32,
        );
        sys::view_dispatcher_run(app.view_dispatcher.as_ptr());
    }

    unsafe {
        let event_queue = sys::furi_message_queue_alloc(8, mem::size_of::<sys::InputEvent>() as u32)
            as *mut sys::FuriMessageQueue;

        // Configure view port
        let view_port = sys::view_port_alloc();
        sys::view_port_draw_callback_set(
            view_port,
            Some(app_draw_callback),
            view_port as *mut c_void,
        );
        sys::view_port_input_callback_set(
            view_port,
            Some(app_input_callback),
            event_queue as *mut c_void,
        );

        // Register view port in GUI
        let gui = UnsafeRecord::open(c"gui".as_ptr());
        sys::gui_add_view_port(gui.as_ptr(), view_port, FULLSCREEN);

        // Assumes draw_callback is defined elsewhere and compatible
        sys::view_port_draw_callback_set(view_port, Some(app_draw_callback), ptr::null_mut());

        let mut event: MaybeUninit<sys::InputEvent> = MaybeUninit::uninit();
        let mut running = true;
        while running {
            if sys::furi_message_queue_get(event_queue, event.as_mut_ptr() as *mut c_void, 100)
                == sys::FuriStatus_FuriStatusOk
            {
                let event = event.assume_init();
                if event.type_ == sys::InputType_InputTypePress
                    || event.type_ == sys::InputType_InputTypeRepeat
                {
                    match event.key {
                        sys::InputKey_InputKeyLeft => IMAGE_POSITION.x -= 2,
                        sys::InputKey_InputKeyRight => IMAGE_POSITION.x += 2,
                        sys::InputKey_InputKeyUp => IMAGE_POSITION.y -= 2,
                        sys::InputKey_InputKeyDown => IMAGE_POSITION.y += 2,
                        _ => running = false,
                    }
                }
            }
            sys::view_port_update(view_port);
        }

        sys::view_port_enabled_set(view_port, false);
        sys::gui_remove_view_port(gui.as_ptr(), view_port);
        sys::view_port_free(view_port);
        sys::furi_message_queue_free(event_queue);
    }

    0
}
