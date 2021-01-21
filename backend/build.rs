use std::{fs, path::Path};

fn main() {
    let dest_path = Path::new("src/image").join("number_image.rs");
    const GROUP_AMOUNT: u8 = 4;

    let mut groups = String::with_capacity(512);
    let mut number_images = String::with_capacity(2048);

    groups.push_str(&format!(
        "pub const NUMBER_IMAGE_GROUPS: [[NumberImage; 10]; {}] = [\n",
        GROUP_AMOUNT
    ));
    for group in 1..(GROUP_AMOUNT + 1) {
        let group_name = &format!("GROUP{}_NUMBERS", group);
        groups.push_str(group_name);
        groups.push_str(",\n");

        number_images.push_str(&format!("pub const {}: [NumberImage; 10] = [\n", group_name));
        for i in 0..10 {
            number_images.push_str("    NumberImage {\n");
            number_images.push_str(&format!(
                "       data: include_bytes!(\"../asset/icon/{}-{}.png\"),\n",
                group, i
            ));
            number_images.push_str("    },\n");
        }
        number_images.push_str("];\n");
    }
    groups.push_str("];\n");

    let mut all = String::with_capacity(groups.len() + number_images.len());
    all.push_str(&groups);
    all.push_str(&number_images);
    fs::write(&dest_path, &all).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
