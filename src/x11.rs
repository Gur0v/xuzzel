use crate::config::Config;
use crate::model::{IconBitmap, MatchResult};
use cairo::{Context, Format, ImageSurface};
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

#[repr(C)]
pub struct Display {
    _private: [u8; 0],
}

pub type Window = c_ulong;
pub type Drawable = c_ulong;
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
    _private: [u8; 0],
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
    fn XSetForeground(display: *mut Display, gc: GC, foreground: c_ulong) -> c_int;
    fn XFillRectangle(
        display: *mut Display,
        drawable: Drawable,
        gc: GC,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
    ) -> c_int;
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
    gc: GC,
    screen: c_int,
    width: u32,
    height: u32,
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
            let border = rgb(config.colors.border) as c_ulong;
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
                border_width,
                border,
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
            request_focus(display, root, window);
            XFlush(display);

            Ok(Self {
                display,
                window,
                gc,
                screen,
                width,
                height,
                row_height,
                content_x: config.horizontal_pad as i32,
                content_width: width.saturating_sub(config.horizontal_pad.saturating_mul(2)),
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
        unsafe {
            XSetForeground(self.display, self.gc, rgb(config.colors.background) as c_ulong);
            XFillRectangle(
                self.display,
                self.window,
                self.gc,
                0,
                0,
                self.width,
                config
                    .vertical_pad
                    .saturating_mul(2)
                    .saturating_add((matches.len() as u32 + 3) * self.row_height),
            );

            let mut cursor_y = config.vertical_pad as i32 + self.baseline_offset;
            if !config.message.is_empty() {
                for line in config.message.lines() {
                    self.draw_text(
                        self.content_x,
                        cursor_y,
                        line,
                        rgb(config.colors.message) as c_ulong,
                        rgb(config.colors.background),
                    );
                    cursor_y += self.row_height as i32 + config.inner_pad as i32;
                }
            }

            if !config.hide_prompt {
                let prompt_width = self.text_width(&config.prompt).max(8);
                self.draw_text(
                    self.content_x,
                    cursor_y,
                    &config.prompt,
                    rgb(config.colors.prompt) as c_ulong,
                    rgb(config.colors.background),
                );

                let display_input = if input.is_empty() && !config.placeholder.is_empty() { config.placeholder.as_str() } else { input };
                let input_color = if input.is_empty() && !config.placeholder.is_empty() {
                    rgb(config.colors.placeholder)
                } else {
                    rgb(config.colors.input)
                };
                self.draw_text(
                    self.content_x + prompt_width + 4,
                    cursor_y,
                    display_input,
                    input_color as c_ulong,
                    rgb(config.colors.background),
                );
                if config.match_counter {
                    let counter = format!("{}/{}", matches.len(), total_matches);
                    let counter_width = self.text_width(&counter);
                    self.draw_text(
                        (self.width as i32 - config.horizontal_pad as i32 - counter_width).max(self.content_x),
                        cursor_y,
                        &counter,
                        rgb(config.colors.counter) as c_ulong,
                        rgb(config.colors.background),
                    );
                }
                cursor_y += self.row_height as i32 + config.inner_pad as i32;
            }

            let row_start = cursor_y - self.baseline_offset;
            for (idx, item) in matches.iter().enumerate() {
                let row_y = row_start + (idx as i32 * self.row_height as i32);
                let active = idx == selected;

                if active {
                    XSetForeground(
                        self.display,
                        self.gc,
                        rgb(config.colors.selection_background) as c_ulong,
                    );
                    XFillRectangle(
                        self.display,
                        self.window,
                        self.gc,
                        self.content_x,
                        row_y,
                        self.content_width,
                        self.row_height.saturating_sub(1),
                    );
                }

                let fg = if active {
                    rgb(config.colors.selection_text) as c_ulong
                } else {
                    rgb(config.colors.text) as c_ulong
                };

                let mut text_x = self.content_x;
                if let Some(icon) = item.entry.icon.as_deref() {
                    self.draw_icon(
                        icon,
                        self.content_x,
                        row_y + ((self.row_height as i32 - self.icon_size as i32) / 2),
                        rgb(config.colors.background),
                    );
                    text_x += self.icon_size as i32 + self.icon_gap;
                }

                self.draw_text(
                    text_x,
                    row_y + self.baseline_offset,
                    &item.entry.label,
                    fg,
                    if active {
                        rgb(config.colors.selection_background)
                    } else {
                        rgb(config.colors.background)
                    },
                );
                if !item.matched_indices.is_empty() {
                    self.draw_match_hint(
                        text_x,
                        &item.entry.label,
                        &item.matched_indices,
                        row_y + self.baseline_offset,
                        if active {
                            rgb(config.colors.selection_background)
                        } else {
                            rgb(config.colors.background)
                        },
                        if active {
                            rgb(config.colors.selection_match) as c_ulong
                        } else {
                            rgb(config.colors.matched_text) as c_ulong
                        },
                    );
                }
            }

            XFlush(self.display);
        }
    }

    fn draw_icon(&self, icon: &IconBitmap, x: i32, y: i32, background: u32) {
        let pixel_count = (icon.width * icon.height) as usize;
        let byte_len = pixel_count * 4;
        unsafe {
            let data = malloc(byte_len) as *mut u8;
            if data.is_null() {
                return;
            }

            let bg_r = ((background >> 16) & 0xff) as u8;
            let bg_g = ((background >> 8) & 0xff) as u8;
            let bg_b = (background & 0xff) as u8;

            for i in 0..pixel_count {
                let src = i * 4;
                let dst = i * 4;
                let r = icon.rgba[src];
                let g = icon.rgba[src + 1];
                let b = icon.rgba[src + 2];
                let a = icon.rgba[src + 3] as u16;

                let blend = |fg: u8, bg: u8| -> u8 {
                    (((fg as u16 * a) + (bg as u16 * (255 - a))) / 255) as u8
                };

                *data.add(dst) = blend(b, bg_b);
                *data.add(dst + 1) = blend(g, bg_g);
                *data.add(dst + 2) = blend(r, bg_r);
                *data.add(dst + 3) = 0;
            }

            let image = XCreateImage(
                self.display,
                XDefaultVisual(self.display, self.screen),
                XDefaultDepth(self.display, self.screen) as c_uint,
                2,
                0,
                data.cast::<c_char>(),
                icon.width,
                icon.height,
                32,
                0,
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
                icon.width,
                icon.height,
            );
            XDestroyImage(image);
        }
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

    fn draw_text(&self, x: i32, baseline_y: i32, text: &str, color: c_ulong, background: u32) {
        let clean = text.replace('\0', "");
        if clean.is_empty() {
            return;
        }

        let mut surface = match self.render_text_surface(&clean, color as u32, background) {
            Some(surface) => surface,
            None => return,
        };
        let y = baseline_y - self.baseline_offset;
        self.blit_surface(&mut surface, x, y);
    }

    fn render_text_surface(
        &self,
        text: &str,
        color: u32,
        background: u32,
    ) -> Option<ImageSurface> {
        let (layout, width, height) = self.make_layout(text)?;
        let surface = ImageSurface::create(
            Format::Rgb24,
            width.max(1),
            height.max(self.row_height as i32),
        )
        .ok()?;
        let context = Context::new(&surface).ok()?;
        paint_background(&context, background);
        set_source_rgb(&context, color);
        context.move_to(0.0, 0.0);
        pangocairo::functions::show_layout(&context, &layout);
        Some(surface)
    }

    fn make_layout(&self, text: &str) -> Option<(pango::Layout, i32, i32)> {
        let surface = ImageSurface::create(Format::Rgb24, 1, 1).ok()?;
        let context = Context::new(&surface).ok()?;
        let layout = pangocairo::functions::create_layout(&context);
        let font = FontDescription::from_string(&self.font_name);
        layout.set_font_description(Some(&font));
        layout.set_text(text);
        let (width, height) = layout.pixel_size();
        Some((layout, width, height))
    }

    fn blit_surface(&self, surface: &mut ImageSurface, x: i32, y: i32) {
        surface.flush();
        let width = surface.width() as u32;
        let height = surface.height() as u32;
        let stride = surface.stride();

        unsafe {
            let Ok(data_view) = surface.data() else {
                return;
            };
            let byte_len = (stride * height as i32) as usize;
            let data = malloc(byte_len) as *mut u8;
            if data.is_null() {
                return;
            }
            ptr::copy_nonoverlapping(data_view.as_ptr(), data, byte_len);

            let image = XCreateImage(
                self.display,
                XDefaultVisual(self.display, self.screen),
                XDefaultDepth(self.display, self.screen) as c_uint,
                2,
                0,
                data.cast::<c_char>(),
                width,
                height,
                32,
                stride,
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

    fn draw_match_hint(
        &self,
        start_x: i32,
        text: &str,
        indices: &[usize],
        baseline: i32,
        background: u32,
        color: c_ulong,
    ) {
        let mut cursor_x = start_x;
        for (idx, ch) in text.chars().enumerate() {
            let glyph = ch.to_string();
            let width = self.text_width(&glyph);
            if indices.contains(&idx) {
                self.draw_text(cursor_x, baseline, &glyph, color, background);
            }
            cursor_x += width.max(8);
        }
    }

    fn text_width(&self, text: &str) -> i32 {
        self.make_layout(&text.replace('\0', ""))
            .map(|(_, width, _)| width.max(8))
            .unwrap_or_else(|| (text.len() as i32) * 8)
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
    let list_start_y = config.vertical_pad as i32
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

fn request_focus(display: *mut Display, root: Window, window: Window) {
    unsafe {
        XRaiseWindow(display, window);
        send_active_window(display, root, window);
        let _ = XGrabKeyboard(
            display,
            window,
            1,
            GRAB_MODE_ASYNC,
            GRAB_MODE_ASYNC,
            CURRENT_TIME,
        );
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
        unsafe {
            let _ = self.screen;
            let _ = self.height;
            XUngrabKeyboard(self.display, CURRENT_TIME);
            XFreeGC(self.display, self.gc);
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

fn paint_background(context: &Context, color: u32) {
    set_source_rgb(context, color);
    let _ = context.paint();
}

fn set_source_rgb(context: &Context, color: u32) {
    let r = ((color >> 16) & 0xff) as f64 / 255.0;
    let g = ((color >> 8) & 0xff) as f64 / 255.0;
    let b = (color & 0xff) as f64 / 255.0;
    context.set_source_rgb(r, g, b);
}
