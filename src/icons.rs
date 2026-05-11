use crate::model::{Entry, IconBitmap};
use freedesktop_icons::lookup;
use once_cell::sync::Lazy;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

static ICON_CACHE: Lazy<Mutex<HashMap<IconCacheKey, Arc<IconBitmap>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct IconCacheKey {
    name: String,
    theme: String,
    size: u32,
}

pub fn attach_icons(entries: &mut [Entry], theme: &str, size: u32) {
    for entry in entries.iter_mut() {
        let Some(icon_name) = entry.icon_name.as_deref() else {
            continue;
        };
        entry.icon = load_icon(icon_name, theme, size);
    }
}

fn load_icon(name: &str, theme: &str, size: u32) -> Option<Arc<IconBitmap>> {
    let requested_size = u16::try_from(size).unwrap_or(u16::MAX);
    let key = IconCacheKey {
        name: name.to_string(),
        theme: theme.to_string(),
        size,
    };

    if let Some(icon) = ICON_CACHE.lock().ok()?.get(&key).cloned() {
        return Some(icon);
    }

    let path = if theme == "default" {
        lookup(name).with_size(requested_size).with_cache().find()
    } else {
        lookup(name)
            .with_size(requested_size)
            .with_theme(theme)
            .with_cache()
            .find()
    }?;

    let icon = Arc::new(load_icon_from_path(&path, size)?);
    let _ = ICON_CACHE.lock().map(|mut cache| cache.insert(key, icon.clone()));
    Some(icon)
}

fn load_icon_from_path(path: &Path, size: u32) -> Option<IconBitmap> {
    match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) if ext == "svg" => load_svg_icon(path, size),
        _ => load_raster_icon(path, size),
    }
}

fn load_raster_icon(path: &Path, size: u32) -> Option<IconBitmap> {
    let image = image::open(path).ok()?;
    let resized = image.resize_exact(size, size, image::imageops::FilterType::Triangle);
    let rgba = resized.to_rgba8().into_raw();

    Some(IconBitmap {
        width: size,
        height: size,
        rgba,
    })
}

fn load_svg_icon(path: &Path, size: u32) -> Option<IconBitmap> {
    let data = fs::read(path).ok()?;
    let options = Options::default();
    let tree = Tree::from_data(&data, &options).ok()?;
    let original_size = tree.size();
    let scale_x = size as f32 / original_size.width();
    let scale_y = size as f32 / original_size.height();

    let mut pixmap = Pixmap::new(size, size)?;
    let mut target = pixmap.as_mut();
    resvg::render(
        &tree,
        Transform::from_scale(scale_x, scale_y),
        &mut target,
    );

    let mut output = Vec::with_capacity((size * size * 4) as usize);
    output.extend_from_slice(pixmap.data());

    Some(IconBitmap {
        width: size,
        height: size,
        rgba: output,
    })
}
