use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    static ref AEEST_MAP: HashMap<String, &'static [u8]> = HashMap::with_capacity(10);
}

