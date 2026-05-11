use crate::config::Config;
use crate::model::{IconBitmap, MatchResult};
use cairo::{Context, Format, ImageSurface, Operator, Surface};
use libc::{c_void, free, malloc};
use pango::FontDescription;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
use std::ptr;

const KEY_PRESS: c_int = 2;
const BUTTON_PRESS: c_int = 4;
const EXPOSE: c_int = 12;
const DESTROY_NOTIFY: c_int = 17;
const MAP_NOTIFY: c_int = 19;
const CLIENT_MESSAGE: c_int = 33;

const KEY_PRESS_MASK: c_long = 1 << 0;
const BUTTON_PRESS_MASK: c_long = 1 << 2;
const EXPOSURE_MASK: c_long = 1 << 15;
const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
const SUBSTRUCTURE_NOTIFY_MASK: c_long = 1 << 19;
const SUBSTRUCTURE_REDIRECT_MASK: c_long = 1 << 20;

const XK_BACKSPACE: c_ulong = 0xff08;
const XK_TAB: c_ulong = 0xff09;
const XK_RETURN: c_ulong = 0xff0d;
const XK_ESCAPE: c_ulong = 0xff1b;
const XK_UP: c_ulong = 0xff52;
const XK_DOWN: c_ulong = 0xff54;
const XK_PAGE_UP: c_ulong = 0xff55;
const XK_PAGE_DOWN: c_ulong = 0xff56;
const XK_HOME: c_ulong = 0xff50;
const XK_END: c_ulong = 0xff57;
const CURRENT_TIME: c_ulong = 0;
const GRAB_MODE_ASYNC: c_int = 1;
const GRAB_SUCCESS: c_int = 0;

#[repr(C)]
pub struct Display {
    _private: [u8; 0],
}

pub type Window = c_ulong;
pub type Drawable = c_ulong;
pub type Pixmap = c_ulong;
pub type GC = *mut std::ffi::c_void;
pub type KeySym = c_ulong;
pub type Atom = c_ulong;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XRectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[repr(C)]
pub struct Visual {
    ext_data: *mut c_void,
    visual_id: c_ulong,
    class: c_int,
    red_mask: c_ulong,
    green_mask: c_ulong,
    blue_mask: c_ulong,
    bits_per_rgb: c_int,
    map_entries: c_int,
}

#[repr(C)]
pub struct XImage {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XKeyEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int,
    pub y: c_int,
    pub x_root: c_int,
    pub y_root: c_int,
    pub state: c_uint,
    pub keycode: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XButtonEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int,
    pub y: c_int,
    pub x_root: c_int,
    pub y_root: c_int,
    pub state: c_uint,
    pub button: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ClientMessageData {
    pub b: [i8; 20],
    pub s: [i16; 10],
    pub l: [c_long; 5],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XClientMessageEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub message_type: Atom,
    pub format: c_int,
    pub data: ClientMessageData,
}

#[repr(C)]
pub union XEvent {
    pub type_: c_int,
    pub xkey: XKeyEvent,
    pub xbutton: XButtonEvent,
    pub xclient: XClientMessageEvent,
    pub pad: [c_long; 24],
}

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(name: *const c_char) -> *mut Display;
    fn XCloseDisplay(display: *mut Display) -> c_int;
    fn XDefaultScreen(display: *mut Display) -> c_int;
    fn XRootWindow(display: *mut Display, screen_number: c_int) -> Window;
    fn XDisplayWidth(display: *mut Display, screen_number: c_int) -> c_int;
    fn XDisplayHeight(display: *mut Display, screen_number: c_int) -> c_int;
    fn XDefaultVisual(display: *mut Display, screen_number: c_int) -> *mut Visual;
    fn XDefaultDepth(display: *mut Display, screen_number: c_int) -> c_int;
    fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        border: c_ulong,
        background: c_ulong,
    ) -> Window;
    fn XStoreName(display: *mut Display, window: Window, name: *const c_char) -> c_int;
    fn XCreatePixmap(
        display: *mut Display,
        drawable: Drawable,
        width: c_uint,
        height: c_uint,
        depth: c_uint,
    ) -> Pixmap;
    fn XFreePixmap(display: *mut Display, pixmap: Pixmap) -> c_int;
    fn XSelectInput(display: *mut Display, window: Window, event_mask: c_long) -> c_int;
    fn XMapRaised(display: *mut Display, window: Window) -> c_int;
    fn XMoveWindow(display: *mut Display, window: Window, x: c_int, y: c_int) -> c_int;
    fn XDestroyWindow(display: *mut Display, window: Window) -> c_int;
    fn XRaiseWindow(display: *mut Display, window: Window) -> c_int;
    fn XGrabKeyboard(
        display: *mut Display,
        grab_window: Window,
        owner_events: c_int,
        pointer_mode: c_int,
        keyboard_mode: c_int,
        time: c_ulong,
    ) -> c_int;
    fn XUngrabKeyboard(display: *mut Display, time: c_ulong) -> c_int;
    fn XInternAtom(display: *mut Display, atom_name: *const c_char, only_if_exists: c_int) -> Atom;
    fn XChangeProperty(
        display: *mut Display,
        window: Window,
        property: Atom,
        type_: Atom,
        format: c_int,
        mode: c_int,
        data: *const u8,
        nelements: c_int,
    ) -> c_int;
    fn XSetTransientForHint(display: *mut Display, window: Window, prop_window: Window) -> c_int;
    fn XCreateGC(
        display: *mut Display,
        drawable: Drawable,
        valuemask: c_ulong,
        values: *mut std::ffi::c_void,
    ) -> GC;
    fn XFreeGC(display: *mut Display, gc: GC) -> c_int;
    fn XSetGraphicsExposures(display: *mut Display, gc: GC, graphics_exposures: c_int) -> c_int;
    fn XCreateImage(
        display: *mut Display,
        visual: *mut Visual,
        depth: c_uint,
        format: c_int,
        offset: c_int,
        data: *mut c_char,
        width: c_uint,
        height: c_uint,
        bitmap_pad: c_int,
        bytes_per_line: c_int,
    ) -> *mut XImage;
    fn XPutImage(
        display: *mut Display,
        drawable: Drawable,
        gc: GC,
        image: *mut XImage,
        src_x: c_int,
        src_y: c_int,
        dest_x: c_int,
        dest_y: c_int,
        width: c_uint,
        height: c_uint,
    ) -> c_int;
    fn XDestroyImage(ximage: *mut XImage) -> c_int;
    fn XFlush(display: *mut Display) -> c_int;
    fn XSync(display: *mut Display, discard: c_int) -> c_int;
    fn XNextEvent(display: *mut Display, event_return: *mut XEvent) -> c_int;
    fn XLookupString(
        event_struct: *mut XKeyEvent,
        buffer_return: *mut c_char,
        bytes_buffer: c_int,
        keysym_return: *mut KeySym,
        status_in_out: *mut std::ffi::c_void,
    ) -> c_int;
    fn XSendEvent(
        display: *mut Display,
        window: Window,
        propagate: c_int,
        event_mask: c_long,
        event_send: *mut XEvent,
    ) -> c_int;
    fn XCopyArea(
        display: *mut Display,
        src: Drawable,
        dest: Drawable,
        gc: GC,
        src_x: c_int,
        src_y: c_int,
        width: c_uint,
        height: c_uint,
        dest_x: c_int,
        dest_y: c_int,
    ) -> c_int;
}

#[link(name = "Xext")]
extern "C" {
    fn XShapeCombineRectangles(
        display: *mut Display,
        dest: Window,
        dest_kind: c_int,
        x_off: c_int,
        y_off: c_int,
        rectangles: *mut XRectangle,
        n_rectangles: c_int,
        op: c_int,
        ordering: c_int,
    );
}

pub enum UiAction {
    SubmitSelected,
    SubmitAt(usize),
    MoveSelection(isize),
    Page(isize),
    JumpToEdge(bool),
    Cancel,
    Continue,
}

pub struct X11Ui {
    display: *mut Display,
    window: Window,
    backbuffer: Pixmap,
    gc: GC,
    cairo_surface: Option<Surface>,
    screen: c_int,
    width: u32,
    height: u32,
    border_width: u32,
    row_height: u32,
    content_x: i32,
    content_width: u32,
    baseline_offset: i32,
    icon_size: u32,
    icon_gap: i32,
    font_name: String,
}

impl X11Ui {
    pub fn open(config: &Config, rows: usize) -> Result<Self, String> {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err("failed to open X11 display".to_string());
            }

            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);
            let background = rgb(config.colors.background) as c_ulong;
            let border_width = config.border.width;
            let prompt_rows = if config.hide_prompt { 0 } else { 1 };
            let message_rows = if config.message.is_empty() {
                0
            } else {
                config.message.lines().count() as u32
            };
            let list_rows = rows.max(1) as u32;
            let (char_width, baseline_offset) = measure_font(&config.font);
            let row_height = config.line_height.unwrap_or(22);
            let icon_size = if config.icons_enabled {
                ((row_height as f32) * config.image_size_ratio).round().max(16.0) as u32
            } else {
                0
            };
            let width = config
                .horizontal_pad
                .saturating_mul(2)
                .saturating_add(config.width.saturating_mul(char_width));
            let height = config
                .vertical_pad
                .saturating_mul(2)
                .saturating_add((prompt_rows + message_rows + list_rows) * row_height)
                .saturating_add(config.inner_pad.saturating_mul(prompt_rows + message_rows));
            let window = XCreateSimpleWindow(
                display,
                root,
                0,
                0,
                width,
                height,
                0,
                0,
                background,
            );
            let title = CString::new("xuzzel").unwrap();
            XStoreName(display, window, title.as_ptr());
            set_floating_hints(display, window, root);
            XSelectInput(
                display,
                window,
                KEY_PRESS_MASK | BUTTON_PRESS_MASK | EXPOSURE_MASK | STRUCTURE_NOTIFY_MASK,
            );
            apply_window_radius(display, window, width, height, config.border.radius);

            let gc = XCreateGC(display, window, 0, ptr::null_mut());
            XSetGraphicsExposures(display, gc, 0);
            let depth = XDefaultDepth(display, screen) as c_uint;
            let backbuffer = XCreatePixmap(display, root, width, height, depth);
            let cairo_surface = Surface::from_raw_full(cairo::ffi::cairo_xlib_surface_create(
                display.cast(),
                backbuffer,
                XDefaultVisual(display, screen).cast(),
                width as c_int,
                height as c_int,
            ))
            .map_err(|err| format!("failed to create cairo xlib surface: {err}"))?;

            let sw = XDisplayWidth(display, screen);
            let sh = XDisplayHeight(display, screen);
            let x = if config.window_x >= 0 {
                config.window_x
            } else {
                ((sw - width as c_int) / 2).max(0)
            };
            let y = if config.window_y >= 0 {
                config.window_y
            } else {
                ((sh as f32 * 0.18) as c_int).max(0)
            };

            XMoveWindow(display, window, x, y);
            XMapRaised(display, window);
            XSync(display, 0);
            let _ = request_focus(display, root, window);
            XFlush(display);

            Ok(Self {
                display,
                window,
                backbuffer,
                gc,
                cairo_surface: Some(cairo_surface),
                screen,
                width,
                height,
                border_width,
                row_height,
                content_x: border_width as i32 + config.horizontal_pad as i32,
                content_width: width
                    .saturating_sub(border_width.saturating_mul(2))
                    .saturating_sub(config.horizontal_pad.saturating_mul(2)),
                baseline_offset: baseline_offset.max(((row_height as i32) / 2) + 5),
                icon_size,
                icon_gap: 10,
                font_name: config.font.clone(),
            })
        }
    }

    pub fn draw(
        &self,
        config: &Config,
        input: &str,
        total_matches: usize,
        matches: &[MatchResult],
        selected: usize,
    ) {
        let surface = match self.cairo_surface.as_ref() {
            Some(surface) => surface,
            None => return,
        };
        let context = match Context::new(surface) {
            Ok(context) => context,
            Err(_) => return,
        };

        self.draw_window_background_on_context(&context, config);

        let mut cursor_y =
            self.border_width as i32 + config.vertical_pad as i32 + self.baseline_offset;
        if !config.message.is_empty() {
            for line in config.message.lines() {
                self.draw_text_on_context(
                    &context,
                    self.content_x,
                    cursor_y,
                    line,
                    rgb(config.colors.message),
                );
                cursor_y += self.row_height as i32 + config.inner_pad as i32;
            }
        }

        if !config.hide_prompt {
            let prompt_width = self.text_width(&config.prompt).max(8);
            self.draw_text_on_context(
                &context,
                self.content_x,
                cursor_y,
                &config.prompt,
                rgb(config.colors.prompt),
            );

            let display_input = if input.is_empty() && !config.placeholder.is_empty() {
                config.placeholder.as_str()
            } else {
                input
            };
            let input_color = if input.is_empty() && !config.placeholder.is_empty() {
                rgb(config.colors.placeholder)
            } else {
                rgb(config.colors.input)
            };
            self.draw_text_on_context(
                &context,
                self.content_x + prompt_width + 4,
                cursor_y,
                display_input,
                input_color,
            );
            if config.match_counter {
                let counter = format!("{}/{}", matches.len(), total_matches);
                let counter_width = self.text_width(&counter);
                self.draw_text_on_context(
                    &context,
                    (self.width as i32 - config.horizontal_pad as i32 - counter_width)
                        .max(self.content_x),
                    cursor_y,
                    &counter,
                    rgb(config.colors.counter),
                );
            }
            cursor_y += self.row_height as i32 + config.inner_pad as i32;
        }

        let row_start = cursor_y - self.baseline_offset;
        for (idx, item) in matches.iter().enumerate() {
            let row_y = row_start + (idx as i32 * self.row_height as i32);
            let active = idx == selected;

            if active {
                set_source_rgb(&context, rgb(config.colors.selection_background));
                context.rectangle(
                    self.content_x as f64,
                    row_y as f64,
                    self.content_width as f64,
                    self.row_height.saturating_sub(1) as f64,
                );
                let _ = context.fill();
            }

            let fg = if active {
                rgb(config.colors.selection_text)
            } else {
                rgb(config.colors.text)
            };

            let mut text_x = self.content_x;
            if let Some(icon) = item.entry.icon.as_deref() {
                self.draw_icon_on_context(
                    &context,
                    icon,
                    self.content_x,
                    row_y + ((self.row_height as i32 - self.icon_size as i32) / 2),
                );
                text_x += self.icon_size as i32 + self.icon_gap;
            }

            self.draw_text_on_context(
                &context,
                text_x,
                row_y + self.baseline_offset,
                &item.entry.label,
                fg,
            );
            if !item.matched_indices.is_empty() {
                self.draw_match_hint_on_context(
                    &context,
                    text_x,
                    &item.entry.label,
                    &item.matched_indices,
                    row_y + self.baseline_offset,
                    if active {
                        rgb(config.colors.selection_match)
                    } else {
                        rgb(config.colors.matched_text)
                    },
                );
            }
        }

        surface.flush();
        unsafe {
            XCopyArea(
                self.display,
                self.backbuffer,
                self.window,
                self.gc,
                0,
                0,
                self.width,
                self.height,
                0,
                0,
            );
        }
        unsafe {
            XFlush(self.display);
        }
    }

    fn draw_text_on_context(
        &self,
        context: &Context,
        x: i32,
        baseline_y: i32,
        text: &str,
        color: u32,
    ) {
        let clean = text.replace('\0', "");
        if clean.is_empty() {
            return;
        }
        let mut surface = match self.render_text_surface(&clean, color) {
            Some(surface) => surface,
            None => return,
        };
        let y = baseline_y - self.baseline_offset;
        self.draw_surface_on_context(context, &mut surface, x, y);
    }

    fn draw_match_hint_on_context(
        &self,
        context: &Context,
        start_x: i32,
        text: &str,
        indices: &[usize],
        baseline: i32,
        color: u32,
    ) {
        let mut cursor_x = start_x;
        for (idx, ch) in text.chars().enumerate() {
            let glyph = ch.to_string();
            let width = self.text_width(&glyph);
            if indices.contains(&idx) {
                self.draw_text_on_context(context, cursor_x, baseline, &glyph, color);
            }
            cursor_x += width.max(8);
        }
    }

    fn draw_icon_on_context(&self, context: &Context, icon: &IconBitmap, x: i32, y: i32) {
        let mut surface = match ImageSurface::create(Format::ARgb32, icon.width as i32, icon.height as i32) {
            Ok(surface) => surface,
            Err(_) => return,
        };
        {
            let stride = surface.stride() as usize;
            let mut data = match surface.data() {
                Ok(data) => data,
                Err(_) => return,
            };
            for row in 0..icon.height as usize {
                for col in 0..icon.width as usize {
                    let src = (row * icon.width as usize + col) * 4;
                    let dst = row * stride + col * 4;
                    let red = icon.rgba[src];
                    let green = icon.rgba[src + 1];
                    let blue = icon.rgba[src + 2];
                    let alpha = icon.rgba[src + 3] as u16;
                    data[dst] = ((blue as u16 * alpha) / 255) as u8;
                    data[dst + 1] = ((green as u16 * alpha) / 255) as u8;
                    data[dst + 2] = ((red as u16 * alpha) / 255) as u8;
                    data[dst + 3] = alpha as u8;
                }
            }
        }
        self.draw_surface_on_context(context, &mut surface, x, y);
    }

    pub fn next_action(
        &self,
        config: &Config,
        selected: usize,
        len: usize,
        input: &mut String,
    ) -> UiAction {
        unsafe {
            let mut event = MaybeUninit::<XEvent>::zeroed();
            XNextEvent(self.display, event.as_mut_ptr());
            let event = event.assume_init();

            match event.type_ {
                EXPOSE => UiAction::Continue,
                MAP_NOTIFY => {
                    let root = XRootWindow(self.display, self.screen);
                    let _ = request_focus(self.display, root, self.window);
                    UiAction::Continue
                }
                DESTROY_NOTIFY => UiAction::Cancel,
                BUTTON_PRESS => {
                    let button = event.xbutton;
                    match button.button {
                        1 => {
                            let index = row_index_from_y(config, button.y, self.row_height);
                            if index < len {
                                UiAction::SubmitAt(index)
                            } else {
                                UiAction::SubmitAt(selected.min(len.saturating_sub(1)))
                            }
                        }
                        4 => UiAction::MoveSelection(-1),
                        5 => UiAction::MoveSelection(1),
                        6 => UiAction::Page(-1),
                        7 => UiAction::Page(1),
                        _ => UiAction::Continue,
                    }
                }
                KEY_PRESS => {
                    let mut key_event = event.xkey;
                    let mut bytes = [0_i8; 32];
                    let mut keysym: KeySym = 0;
                    let len_read = XLookupString(
                        &mut key_event,
                        bytes.as_mut_ptr(),
                        bytes.len() as c_int,
                        &mut keysym,
                        ptr::null_mut(),
                    );

                    match keysym {
                        XK_ESCAPE => UiAction::Cancel,
                        XK_RETURN => UiAction::SubmitSelected,
                        XK_BACKSPACE => {
                            input.pop();
                            UiAction::Continue
                        }
                        XK_HOME => UiAction::JumpToEdge(true),
                        XK_END => UiAction::JumpToEdge(false),
                        XK_UP => UiAction::MoveSelection(-1),
                        XK_DOWN | XK_TAB => UiAction::MoveSelection(1),
                        XK_PAGE_UP => UiAction::Page(-1),
                        XK_PAGE_DOWN => UiAction::Page(1),
                        _ => {
                            if len_read > 0 {
                                let text = bytes[..len_read as usize]
                                    .iter()
                                    .map(|b| *b as u8 as char)
                                    .collect::<String>();
                                if text.chars().all(|ch| !ch.is_control()) {
                                    input.push_str(&text);
                                }
                            }
                            UiAction::Continue
                        }
                    }
                }
                _ => UiAction::Continue,
            }
        }
    }

    fn make_layout_for_context(&self, context: &Context, text: &str) -> Option<pango::Layout> {
        let layout = pangocairo::functions::create_layout(context);
        let font = FontDescription::from_string(&self.font_name);
        layout.set_font_description(Some(&font));
        layout.set_text(text);
        Some(layout)
    }

    fn render_text_surface(&self, text: &str, color: u32) -> Option<ImageSurface> {
        let (layout, width, height) = self.make_layout(text)?;
        let surface = ImageSurface::create(
            Format::ARgb32,
            width.max(1),
            height.max(self.row_height as i32),
        )
        .ok()?;
        let context = Context::new(&surface).ok()?;
        context.set_operator(Operator::Source);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        let _ = context.paint();
        context.set_operator(Operator::Over);
        set_source_rgb(&context, color);
        context.move_to(0.0, 0.0);
        pangocairo::functions::show_layout(&context, &layout);
        Some(surface)
    }

    fn make_layout(&self, text: &str) -> Option<(pango::Layout, i32, i32)> {
        let surface = ImageSurface::create(Format::Rgb24, 1, 1).ok()?;
        let context = Context::new(&surface).ok()?;
        let layout = self.make_layout_for_context(&context, text)?;
        let (width, height) = layout.pixel_size();
        Some((layout, width, height))
    }

    fn draw_surface_on_context(
        &self,
        context: &Context,
        surface: &mut ImageSurface,
        x: i32,
        y: i32,
    ) {
        let width = surface.width() as f64;
        let height = surface.height() as f64;
        surface.flush();
        let _ = context.save();
        context.rectangle(x as f64, y as f64, width, height);
        context.clip();
        let _ = context.set_source_surface(surface, x as f64, y as f64);
        let _ = context.paint();
        let _ = context.restore();
    }

    fn blit_surface(&self, surface: &mut ImageSurface, x: i32, y: i32, background: Option<u32>) {
        surface.flush();
        let width = surface.width() as u32;
        let height = surface.height() as u32;
        let stride = surface.stride() as usize;

        unsafe {
            let Ok(data_view) = surface.data() else {
                return;
            };
            let visual = XDefaultVisual(self.display, self.screen);
            if visual.is_null() {
                return;
            }

            let byte_len = (width * height * 4) as usize;
            let data = malloc(byte_len) as *mut u8;
            if data.is_null() {
                return;
            }

            let red_mask = (*visual).red_mask as u32;
            let green_mask = (*visual).green_mask as u32;
            let blue_mask = (*visual).blue_mask as u32;

            for row in 0..height as usize {
                for col in 0..width as usize {
                    let src = row * stride + col * 4;
                    let dst = (row * width as usize + col) * 4;
                    let blue = data_view[src];
                    let green = data_view[src + 1];
                    let red = data_view[src + 2];
                    let alpha = data_view[src + 3];
                    let (red, green, blue) = if let Some(bg) = background {
                        let bg_red = ((bg >> 16) & 0xff) as u8;
                        let bg_green = ((bg >> 8) & 0xff) as u8;
                        let bg_blue = (bg & 0xff) as u8;
                        blend_pixel(red, green, blue, alpha, bg_red, bg_green, bg_blue)
                    } else {
                        (red, green, blue)
                    };
                    let pixel = pack_visual_pixel(red, green, blue, red_mask, green_mask, blue_mask);
                    let bytes = pixel.to_ne_bytes();
                    *data.add(dst) = bytes[0];
                    *data.add(dst + 1) = bytes[1];
                    *data.add(dst + 2) = bytes[2];
                    *data.add(dst + 3) = bytes[3];
                }
            }

            let image = XCreateImage(
                self.display,
                visual,
                XDefaultDepth(self.display, self.screen) as c_uint,
                2,
                0,
                data.cast::<c_char>(),
                width,
                height,
                32,
                (width * 4) as c_int,
            );

            if image.is_null() {
                free(data.cast::<c_void>());
                return;
            }

            XPutImage(
                self.display,
                self.window,
                self.gc,
                image,
                0,
                0,
                x,
                y,
                width,
                height,
            );
            XDestroyImage(image);
        }
    }

    fn text_width(&self, text: &str) -> i32 {
        self.make_layout(&text.replace('\0', ""))
            .map(|(_, width, _)| width.max(8))
            .unwrap_or_else(|| (text.len() as i32) * 8)
    }

    fn draw_window_background_on_context(&self, context: &Context, config: &Config) {
        let outer_radius = config.border.radius.min(self.width / 2).min(self.height / 2) as f64;
        draw_rounded_rect(
            context,
            0.0,
            0.0,
            self.width as f64,
            self.height as f64,
            outer_radius,
        );
        set_source_rgb(context, rgb(config.colors.border));
        let _ = context.fill();

        let border = self.border_width as f64;
        let inner_width = (self.width as f64 - border * 2.0).max(0.0);
        let inner_height = (self.height as f64 - border * 2.0).max(0.0);
        if inner_width > 0.0 && inner_height > 0.0 {
            let inner_radius = config
                .border
                .radius
                .saturating_sub(self.border_width)
                .min(inner_width as u32 / 2)
                .min(inner_height as u32 / 2) as f64;
            draw_rounded_rect(
                context,
                border,
                border,
                inner_width,
                inner_height,
                inner_radius,
            );
            set_source_rgb(context, rgb(config.colors.background));
            let _ = context.fill();
        }
    }
}

fn row_index_from_y(config: &Config, y: i32, row_height: u32) -> usize {
    let message_rows = if config.message.is_empty() {
        0
    } else {
        config.message.lines().count() as u32
    };
    let prompt_rows: u32 = if config.hide_prompt { 0 } else { 1 };
    let inner_gaps = message_rows + prompt_rows.saturating_sub(1);
    let list_start_y = config.border.width as i32
        + config.vertical_pad as i32
        + ((message_rows + prompt_rows) * row_height) as i32
        + (inner_gaps * config.inner_pad) as i32;
    if y < list_start_y {
        return usize::MAX;
    }

    ((y - list_start_y) as u32 / row_height) as usize
}

fn set_floating_hints(display: *mut Display, window: Window, root: Window) {
    unsafe {
        const PROP_MODE_REPLACE: c_int = 0;
        const FALSE: c_int = 0;

        let atom = |name: &str| -> Option<Atom> {
            let name = CString::new(name).ok()?;
            let atom = XInternAtom(display, name.as_ptr(), FALSE);
            if atom == 0 {
                None
            } else {
                Some(atom)
            }
        };

        let Some(xa_atom) = atom("ATOM") else {
            return;
        };
        let Some(net_wm_window_type) = atom("_NET_WM_WINDOW_TYPE") else {
            return;
        };
        let Some(net_wm_window_type_dialog) = atom("_NET_WM_WINDOW_TYPE_DIALOG") else {
            return;
        };
        let Some(net_wm_window_type_utility) = atom("_NET_WM_WINDOW_TYPE_UTILITY") else {
            return;
        };
        let Some(net_wm_state) = atom("_NET_WM_STATE") else {
            return;
        };
        let Some(net_wm_state_above) = atom("_NET_WM_STATE_ABOVE") else {
            return;
        };
        let Some(net_wm_state_modal) = atom("_NET_WM_STATE_MODAL") else {
            return;
        };
        let Some(net_wm_state_skip_taskbar) = atom("_NET_WM_STATE_SKIP_TASKBAR") else {
            return;
        };
        let Some(net_wm_state_skip_pager) = atom("_NET_WM_STATE_SKIP_PAGER") else {
            return;
        };

        let window_types = [net_wm_window_type_dialog, net_wm_window_type_utility];
        XChangeProperty(
            display,
            window,
            net_wm_window_type,
            xa_atom,
            32,
            PROP_MODE_REPLACE,
            window_types.as_ptr().cast::<u8>(),
            window_types.len() as c_int,
        );

        let window_states = [
            net_wm_state_above,
            net_wm_state_modal,
            net_wm_state_skip_taskbar,
            net_wm_state_skip_pager,
        ];
        XChangeProperty(
            display,
            window,
            net_wm_state,
            xa_atom,
            32,
            PROP_MODE_REPLACE,
            window_states.as_ptr().cast::<u8>(),
            window_states.len() as c_int,
        );

        XSetTransientForHint(display, window, root);
    }
}

fn request_focus(display: *mut Display, root: Window, window: Window) -> bool {
    unsafe {
        XRaiseWindow(display, window);
        send_active_window(display, root, window);
        XSync(display, 0);
        XGrabKeyboard(
            display,
            window,
            1,
            GRAB_MODE_ASYNC,
            GRAB_MODE_ASYNC,
            CURRENT_TIME,
        ) == GRAB_SUCCESS
    }
}

fn send_active_window(display: *mut Display, root: Window, window: Window) {
    unsafe {
        const FALSE: c_int = 0;

        let atom = |name: &str| -> Option<Atom> {
            let name = CString::new(name).ok()?;
            let atom = XInternAtom(display, name.as_ptr(), FALSE);
            if atom == 0 {
                None
            } else {
                Some(atom)
            }
        };

        let Some(net_active_window) = atom("_NET_ACTIVE_WINDOW") else {
            return;
        };

        let mut event = XEvent {
            xclient: XClientMessageEvent {
                type_: CLIENT_MESSAGE,
                serial: 0,
                send_event: 1,
                display,
                window,
                message_type: net_active_window,
                format: 32,
                data: ClientMessageData {
                    l: [1, CURRENT_TIME as c_long, 0, 0, 0],
                },
            },
        };

        XSendEvent(
            display,
            root,
            FALSE,
            SUBSTRUCTURE_NOTIFY_MASK | SUBSTRUCTURE_REDIRECT_MASK,
            &mut event,
        );
    }
}

impl Drop for X11Ui {
    fn drop(&mut self) {
        if let Some(surface) = self.cairo_surface.take() {
            surface.finish();
            drop(surface);
        }
        unsafe {
            let _ = self.screen;
            let _ = self.height;
            XUngrabKeyboard(self.display, CURRENT_TIME);
            XFreeGC(self.display, self.gc);
            XFreePixmap(self.display, self.backbuffer);
            XDestroyWindow(self.display, self.window);
            XCloseDisplay(self.display);
        }
    }
}

fn rgb(color: u32) -> u32 {
    color & 0x00ff_ffff
}

fn apply_window_radius(display: *mut Display, window: Window, width: u32, height: u32, radius: u32) {
    const SHAPE_BOUNDING: c_int = 0;
    const SHAPE_SET: c_int = 0;
    const YX_BANDED: c_int = 0;

    let mut rects = rounded_rectangles(width, height, radius);
    if rects.is_empty() {
        return;
    }

    unsafe {
        XShapeCombineRectangles(
            display,
            window,
            SHAPE_BOUNDING,
            0,
            0,
            rects.as_mut_ptr(),
            rects.len() as c_int,
            SHAPE_SET,
            YX_BANDED,
        );
    }
}

fn rounded_rectangles(width: u32, height: u32, radius: u32) -> Vec<XRectangle> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let radius = radius.min(width / 2).min(height / 2);
    if radius == 0 {
        return vec![XRectangle {
            x: 0,
            y: 0,
            width: width.min(u16::MAX as u32) as u16,
            height: height.min(u16::MAX as u32) as u16,
        }];
    }

    let mut rects = Vec::with_capacity(height as usize);
    for y in 0..height {
        let inset = corner_inset(y, height, radius);
        let span_width = width.saturating_sub(inset.saturating_mul(2));
        if span_width == 0 {
            continue;
        }

        rects.push(XRectangle {
            x: inset.min(i16::MAX as u32) as i16,
            y: y.min(i16::MAX as u32) as i16,
            width: span_width.min(u16::MAX as u32) as u16,
            height: 1,
        });
    }

    rects
}

fn corner_inset(y: u32, height: u32, radius: u32) -> u32 {
    let mirrored = y.min(height.saturating_sub(1).saturating_sub(y));
    if mirrored >= radius {
        return 0;
    }

    let r = radius as f64;
    let dy = r - mirrored as f64 - 0.5;
    let dx = (r * r - dy * dy).max(0.0).sqrt();
    (r - dx).ceil().max(0.0) as u32
}

fn measure_font(font_name: &str) -> (u32, i32) {
    let surface = match ImageSurface::create(Format::Rgb24, 1, 1) {
        Ok(surface) => surface,
        Err(_) => return (8, 16),
    };
    let context = match Context::new(&surface) {
        Ok(context) => context,
        Err(_) => return (8, 16),
    };
    let layout = pangocairo::functions::create_layout(&context);
    let font = FontDescription::from_string(font_name);
    layout.set_font_description(Some(&font));
    layout.set_text("0");
    let (width, _) = layout.pixel_size();
    let baseline = layout.baseline() / pango::SCALE;
    (width.max(8) as u32, baseline.max(16))
}

fn set_source_rgb(context: &Context, color: u32) {
    let r = ((color >> 16) & 0xff) as f64 / 255.0;
    let g = ((color >> 8) & 0xff) as f64 / 255.0;
    let b = (color & 0xff) as f64 / 255.0;
    context.set_source_rgb(r, g, b);
}

fn pack_visual_pixel(red: u8, green: u8, blue: u8, red_mask: u32, green_mask: u32, blue_mask: u32) -> u32 {
    scale_channel(red, red_mask) | scale_channel(green, green_mask) | scale_channel(blue, blue_mask)
}

fn blend_pixel(
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
    bg_red: u8,
    bg_green: u8,
    bg_blue: u8,
) -> (u8, u8, u8) {
    if alpha == 255 {
        return (red, green, blue);
    }
    if alpha == 0 {
        return (bg_red, bg_green, bg_blue);
    }

    let alpha = alpha as u16;
    let blend = |fg: u8, bg: u8| -> u8 {
        (((fg as u16 * alpha) + (bg as u16 * (255 - alpha))) / 255) as u8
    };
    (
        blend(red, bg_red),
        blend(green, bg_green),
        blend(blue, bg_blue),
    )
}

fn scale_channel(channel: u8, mask: u32) -> u32 {
    if mask == 0 {
        return 0;
    }

    let shift = mask.trailing_zeros();
    let bits = (mask >> shift).count_ones();
    let max_value = (1u32 << bits) - 1;
    let scaled = (channel as u32 * max_value + 127) / 255;
    (scaled << shift) & mask
}

fn draw_rounded_rect(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let radius = radius.min(width / 2.0).min(height / 2.0).max(0.0);
    if radius == 0.0 {
        context.rectangle(x, y, width, height);
        return;
    }

    let right = x + width;
    let bottom = y + height;
    let quarter = std::f64::consts::FRAC_PI_2;

    context.new_sub_path();
    context.arc(right - radius, y + radius, radius, -quarter, 0.0);
    context.arc(right - radius, bottom - radius, radius, 0.0, quarter);
    context.arc(x + radius, bottom - radius, radius, quarter, 2.0 * quarter);
    context.arc(x + radius, y + radius, radius, 2.0 * quarter, 3.0 * quarter);
    context.close_path();
}
