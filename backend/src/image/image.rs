use std::{io::Write, path::Path, vec::Vec};

use bytes::{buf::BufMut, Bytes, BytesMut};
use image::{
    self,
    codecs::{
        jpeg::JpegEncoder,
        png::{CompressionType, FilterType, PngEncoder},
    },
    ColorType, DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, Luma, Rgb, Rgba, RgbaImage,
};
use rand::{thread_rng, Rng};
use tokio::fs::copy;

use blog_common::{dto::UploadFileInfo, result::Error};

use crate::util::{result::Result, val};

pub type ImageWidth = u32;
pub type ImageHeight = u32;

const MAX_DIMENSION: u32 = 1000;

/*
https://stackoverflow.com/questions/35488820/how-to-create-a-rust-struct-with-an-imageimagebuffer-as-a-member
*/

fn err(image_error: image::error::ImageError) {
    match image_error {
        image::error::ImageError::Decoding(de) => {
            dbg!(de);
        },
        image::error::ImageError::Encoding(ee) => {
            dbg!(ee);
        },
        image::error::ImageError::Parameter(pe) => {
            dbg!(pe);
        },
        image::error::ImageError::Limits(le) => {
            dbg!(le);
        },
        image::error::ImageError::Unsupported(ue) => {
            dbg!(ue);
        },
        image::error::ImageError::IoError(e) => {
            dbg!(e);
        },
    };
}

pub fn gen_verify_image(numbers: &[u8]) -> Bytes {
    let number_len = numbers.len() as u32;
    const WIDTH: u32 = 64u32;
    const HEIGHT: u32 = 64u32;
    let width = number_len * WIDTH;
    // let mut img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(width, height, |x, y| {
    //     if x % 2 == 0 || y % 5 == 0 {
    //         Luma([0u8])
    //     } else {
    //         Luma([255u8])
    //     }
    // });
    let mut img = RgbaImage::new(width, HEIGHT);
    // let raw_data = img.into_raw();
    // let data = raw_data.as_slice();
    // dbg!(data);

    let mut x_offset = 0u32;
    for n in numbers.into_iter() {
        let number = image::load_from_memory_with_format(
            super::asset::rand_group_number_image(*n as usize).data,
            image::ImageFormat::Png,
        )
        .unwrap();
        let mut rng = thread_rng();
        for (x, y, pixel) in number.to_rgba8().enumerate_pixels() {
            // pixel.0[3] = 75;
            if x % 10 == 0 || y % 10 == 0 {
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                        100,
                    ]),
                );
            } else {
                img.put_pixel(x + x_offset, y, *pixel);
            }
        }
        x_offset += WIDTH;
    }

    let mut b = BytesMut::with_capacity(16384).writer();
    // let mut encoder = JpegEncoder::new_with_quality(&mut out, 70);
    // let r = encoder.encode_image(&img);
    let encoder = PngEncoder::new_with_quality(&mut b, CompressionType::Default, FilterType::NoFilter);
    encoder.encode(&img.into_raw(), width, HEIGHT, ColorType::Rgba8);
    // dbg!(out.len());
    b.into_inner().freeze()
}

pub async fn resize_from_file(file: &UploadFileInfo) -> Result<()> {
    let image_format = match file.extension.as_str() {
        "gif" => ImageFormat::Gif,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        _ => return Err(Error::UnsupportedFileType(file.extension.clone()).into()),
    };

    let filepath = file.filepath.as_path();

    let (mut w, mut h) = match image::image_dimensions(filepath) {
        Ok((w, h)) => (w, h),
        Err(e) => {
            dbg!(e);
            return Err(Error::UnknownFileType.into());
        },
    };

    if h <= MAX_DIMENSION && w <= MAX_DIMENSION {
        return Ok(())
    }

    let dynamic_image = match image::open(filepath) {
        Ok(i) => i,
        Err(e) => {
            err(e);
            return Err(Error::UnknownFileType.into());
        },
    };

    if w == h {
        w = MAX_DIMENSION;
        h = MAX_DIMENSION;
    } else if w > h {
        h = h * MAX_DIMENSION / w;
        w = MAX_DIMENSION;
    } else {
        w = w * MAX_DIMENSION / h;
        h = MAX_DIMENSION;
    }

    let d = dynamic_image.thumbnail_exact(w, h);
    if let Err(e) = d.save_with_format(&filepath, image_format) {
        dbg!(e);
        return Err(Error::CreateThumbnailFailed.into());
    }

    Ok(())
}

// 下面这个，如果写成：B, 'a，就会提示找不到生命周期
pub fn resize_from_bytes<'a, B>(src_bytes: B) -> Result<()>
where
    B: AsRef<&'a [u8]>,
{
    let image_type = match image::guess_format(src_bytes.as_ref()) {
        Ok(t) => t,
        Err(e) => {
            err(e);
            return Err(Error::UnknownFileType.into());
        },
    };

    let dynamic_image = match image::load_from_memory_with_format(src_bytes.as_ref(), image_type) {
        Ok(i) => i,
        Err(e) => {
            err(e);
            return Err(Error::UnknownFileType.into());
        },
    };

    Ok(())
}
