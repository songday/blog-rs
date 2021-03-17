use std::collections::HashMap;
use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use lazy_static::lazy_static;

// const ALL_THE_FILES: &[(&str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/all_the_files.rs"));
const ALL_ASSET_FILES: &[(&str, &[u8])] = &include!("asset_list.rs");

lazy_static! {
    static ref AEEST_MAP: HashMap<&'static str, &'static [u8]> = {
        let cache: Vec<u8> = Vec::with_capacity(65535);
        let mut asset = HashMap::with_capacity(10);
        for (name, data) in ALL_ASSET_FILES {
            // let mut e = GzEncoder::new(cache, Compression::default());
            // e.write_all(b);
            // let compressed_bytes = e.finish().unwrap();
            // asset.insert(name, compressed_bytes.as_slice());
            asset.insert(*name, *data);
            // asset.insert("", &b[..]);
        }
        asset
    };
}
