use std::io::Write;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageReader};
use sha2::{Digest, Sha256};

use crate::db::{self, IndexedFile};
use crate::state::AppState;

const THUMB_EDGE: u32 = 160;
const JPEG_QUALITY: u8 = 80;
const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

/// Build or read a thumbnail. The cache file is only created under the
/// app-local thumbs directory.
pub fn ensure_jpeg(state: &AppState, path: &str) -> Result<Vec<u8>, String> {
    let indexed = state
        .with_db(|conn| db::indexed_file(conn, path))?
        .ok_or_else(|| "Image is not in the index.".to_string())?;
    if indexed.missing {
        return Err("Image is missing from disk.".into());
    }
    if indexed.size < 0 || indexed.size as u64 > MAX_SOURCE_BYTES {
        return Err("Image is too large to thumbnail.".into());
    }

    let file_name = thumb_file_name(path);
    let cache_path = thumb_cache_path(&state.thumbs_dir, &file_name)?;
    if let Some(bytes) = read_fresh_cache(state, path, &indexed, &cache_path)? {
        return Ok(bytes);
    }

    let jpeg = render_jpeg(Path::new(path))?;
    let temp_path = cache_path.with_extension("jpg.part");
    write_cache(&temp_path, &cache_path, &jpeg.bytes)?;
    state.with_db(|conn| {
        db::save_thumb(
            conn,
            path,
            &file_name,
            jpeg.width,
            jpeg.height,
            indexed.mtime,
            indexed.size,
        )
    })?;
    Ok(jpeg.bytes)
}

pub fn read_original(state: &AppState, path: &str) -> Result<(&'static str, Vec<u8>), String> {
    let indexed = state
        .with_db(|conn| db::indexed_file(conn, path))?
        .ok_or_else(|| "Image is not in the index.".to_string())?;
    if indexed.missing {
        return Err("Image is missing from disk.".into());
    }
    if indexed.size < 0 || indexed.size as u64 > MAX_SOURCE_BYTES {
        return Err("Image is too large to preview.".into());
    }
    let bytes = std::fs::read(path).map_err(|err| err.to_string())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Image is too large to preview.".into());
    }
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    Ok((content_type(&extension), bytes))
}

struct RenderedJpeg {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

fn render_jpeg(path: &Path) -> Result<RenderedJpeg, String> {
    let image = ImageReader::open(path)
        .map_err(|err| err.to_string())?
        .with_guessed_format()
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?;
    if image.width() > 16_000 || image.height() > 16_000 {
        return Err("Image is too large to thumbnail.".into());
    }
    let thumb = image.thumbnail(THUMB_EDGE, THUMB_EDGE);
    let rgb = thumb.to_rgb8();
    let mut bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|err| err.to_string())?;
    Ok(RenderedJpeg {
        bytes,
        width: rgb.width(),
        height: rgb.height(),
    })
}

fn read_fresh_cache(
    state: &AppState,
    path: &str,
    indexed: &IndexedFile,
    expected: &Path,
) -> Result<Option<Vec<u8>>, String> {
    let row = state.with_db(|conn| db::thumb_row(conn, path))?;
    let Some(row) = row else {
        return Ok(None);
    };
    if row.source_mtime != indexed.mtime || row.source_size != indexed.size {
        return Ok(None);
    }
    let cache_path = thumb_cache_path(&state.thumbs_dir, &row.file_name)?;
    if cache_path != expected {
        return Ok(None);
    }
    match std::fs::read(&cache_path) {
        Ok(bytes) if !bytes.is_empty() => Ok(Some(bytes)),
        _ => Ok(None),
    }
}

fn write_cache(temp_path: &Path, final_path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    {
        let mut file = std::fs::File::create(temp_path).map_err(|err| err.to_string())?;
        file.write_all(bytes).map_err(|err| err.to_string())?;
    }
    std::fs::rename(temp_path, final_path).map_err(|err| err.to_string())?;
    Ok(())
}

fn thumb_file_name(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let digest = hasher.finalize();
    let mut name = String::with_capacity(68);
    for byte in digest.as_slice() {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        name.push(HEX[(byte >> 4) as usize] as char);
        name.push(HEX[(byte & 0x0f) as usize] as char);
    }
    name.push_str(".jpg");
    name
}

fn thumb_cache_path(thumbs_dir: &Path, file_name: &str) -> Result<PathBuf, String> {
    if file_name.len() != 68 || !file_name.ends_with(".jpg") {
        return Err("Thumbnail name is not a cache file.".into());
    }
    if !file_name.as_bytes()[..64]
        .iter()
        .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("Thumbnail name is not a cache file.".into());
    }
    let path = thumbs_dir.join(file_name);
    if !path.starts_with(thumbs_dir) {
        return Err("Thumbnail path left the cache directory.".into());
    }
    Ok(path)
}

pub fn content_type(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}
