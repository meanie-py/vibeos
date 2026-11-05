//! QEMU's binary PPM to compressed RGB PNG, without a palette conversion.
use crate::{Error, Result};
use std::{fs, io::Read, path::Path};

pub fn load_rgb(source: &Path) -> Result<(u32, u32, Vec<u8>)> {
    let mut bytes = Vec::new();
    fs::File::open(source)?
        .take(512 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 512 * 1024 * 1024 {
        return Err("QEMU screenshot exceeds 512 MiB".into());
    }
    let mut at = 0;
    fn token<'a>(bytes: &'a [u8], at: &mut usize) -> Result<&'a [u8]> {
        loop {
            while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
                *at += 1;
            }
            if bytes.get(*at) == Some(&b'#') {
                while bytes.get(*at).is_some_and(|b| *b != b'\n') {
                    *at += 1;
                }
            } else {
                break;
            }
        }
        let start = *at;
        while bytes.get(*at).is_some_and(|b| !b.is_ascii_whitespace()) {
            *at += 1;
        }
        if start == *at {
            return Err("truncated PPM header".into());
        }
        Ok(&bytes[start..*at])
    }
    if token(&bytes, &mut at)? != b"P6" {
        return Err("QEMU screenshot is not binary RGB PPM".into());
    }
    fn number(bytes: &[u8]) -> Result<u32> {
        std::str::from_utf8(bytes)
            .map_err(|_| Error::new("invalid PPM number"))?
            .parse()
            .map_err(|_| Error::new("invalid PPM number"))
    }
    let width = number(token(&bytes, &mut at)?)?;
    let height = number(token(&bytes, &mut at)?)?;
    if width == 0 || height == 0 || number(token(&bytes, &mut at)?)? != 255 {
        return Err("PPM requires nonzero dimensions and 8-bit RGB channels".into());
    }
    // Consume the header's one separator; whitespace-valued pixels are data.
    if !bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        return Err("missing PPM separator".into());
    }
    if bytes[at..].starts_with(b"\r\n") {
        at += 2;
    } else {
        at += 1;
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(3))
        .ok_or("PPM dimensions overflow")?;
    if bytes.len().checked_sub(at) != Some(expected) {
        return Err("PPM pixel data size does not match its header".into());
    }
    Ok((width, height, bytes[at..].to_vec()))
}

pub fn convert(source: &Path, destination: &Path) -> Result<()> {
    let (width, height, pixels) = load_rgb(source)?;
    let parent = destination
        .parent()
        .ok_or("PNG destination has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut encoder = png::Encoder::new(temporary.as_file_mut(), width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder
            .write_header()
            .map_err(|e| Error::new(format!("PNG header: {e}")))?;
        writer
            .write_image_data(&pixels)
            .map_err(|e| Error::new(format!("PNG pixels: {e}")))?;
    }
    temporary.as_file().sync_all()?;
    temporary
        .persist(destination)
        .map_err(|e| Error::new(format!("publish screenshot: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_whitespace_pixels_and_refuses_truncation() {
        let dir = tempfile::tempdir().unwrap();
        let ppm = dir.path().join("screen.ppm");
        let png = dir.path().join("screen.png");
        fs::write(&ppm, b"P6\n# comment\n1 1\n255\n\n \r").unwrap();
        convert(&ppm, &png).unwrap();
        let decoder = png::Decoder::new(fs::File::open(&png).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut out = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut out).unwrap();
        assert_eq!(&out[..info.buffer_size()], b"\n \r");
        let saved = fs::read(&png).unwrap();
        fs::write(&ppm, b"P6\n1 1\n255\nX").unwrap();
        assert!(convert(&ppm, &png).is_err());
        assert_eq!(fs::read(&png).unwrap(), saved);
    }
}
