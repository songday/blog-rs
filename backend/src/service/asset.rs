use std::{collections::HashMap, io::Write};

use lazy_static::lazy_static;

// const ALL_THE_FILES: &[(&str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/all_the_files.rs"));
const ALL_ASSET_FILES: &[(&str, &[u8])] = &include!("asset_list.rs");

lazy_static! {
    static ref AEEST_MAP: HashMap<&'static str, &'static [u8]> = {
        let mut asset = HashMap::with_capacity(10);
        for (name, data) in ALL_ASSET_FILES {
            asset.insert(*name, *data);
            // asset.insert("", &b[..]);
        }
        asset
    };
}
