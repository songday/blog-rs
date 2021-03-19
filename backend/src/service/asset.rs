use std::{collections::HashMap, io::Write};

use lazy_static::lazy_static;

// const ALL_THE_FILES: &[(&str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/all_the_files.rs"));
const ALL_ASSET_FILES: &[(&str, &[u8])] = &include!("asset_list.rs");

lazy_static! {
    static ref AEEST_MAP: HashMap<&'static str, usize> = {
        let mut asset = HashMap::with_capacity(10);
        let mut idx = 0usize;
        for (name, _data) in ALL_ASSET_FILES {
            asset.insert(*name, idx);
            // asset.insert("", &b[..]);
            idx += 1;
        }
        asset
    };
}

pub(crate) fn get_asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    let idx = AEEST_MAP.get(path);
    if idx.is_none() {
        return None;
    }
    Some(ALL_ASSET_FILES[*idx.unwrap()])
}

pub(crate) fn get_content_type(filename: &str) -> String {
    if filename.rfind(".css").is_some() {
        String::from("text/css")
    } else if filename.rfind(".js").is_some() {
        String::from("text/javascript")
    } else {
        String::new()
    }
}
