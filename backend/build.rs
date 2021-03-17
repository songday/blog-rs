use core::result::Result;
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

fn walk_assets(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    let mut other_files = walk_assets(entry.path())?;
                    files.append(&mut other_files);
                } else if file_type.is_file() {
                    files.push(entry.path());
                    // files.push(entry.file_name().to_os_string().into_string().unwrap());
                }
            }
        }
    }
    Ok(files)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    // embed all static resource asset files
    let asset_root = format!("{}", Path::new("src").join("resource").join("asset").display());
    let asset_root = asset_root.as_str();
    let all_static_asset_files = walk_assets(asset_root).unwrap();
    let mut service_asset_file = File::create(Path::new("src").join("service").join("asset_list.rs"))?;
    writeln!(&mut service_asset_file, r##"["##,)?;
    for f in all_static_asset_files.iter() {
        writeln!(
            &mut service_asset_file,
            r##"("{name}", include_bytes!(r#"{file_path}"#)),"##,
            name = format!("{}", f.display()).replace(asset_root, "").replace("\\", "/"),
            file_path = format!("{}", f.display()).replace("src", ".."),
        )?;
    }
    writeln!(&mut service_asset_file, r##"]"##,)?;

    // embed images for validate image
    let dest_path = Path::new("src").join("image").join("number_image.rs");
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
                "       data: include_bytes!(\"../resource/icon/{}-{}.png\"),\n",
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

    Ok(())
}
