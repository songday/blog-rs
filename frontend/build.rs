use std::{fs, path::Path};

struct RequestUrls(String);

impl RequestUrls {
    fn new() -> Self { RequestUrls(String::with_capacity(2048)) }
    fn append(&mut self, var_name: &str, uri: &str) {
        self.0.push_str("pub(crate) const ");
        self.0.push_str(var_name);
        self.0.push_str(": &str = \"");
        // self.0.push_str("http://127.0.0.1:9270/");
        // self.0.push_str("https://www.songday.com/");
        self.0.push_str(uri);
        self.0.push_str("\";\n");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let dest_path = Path::new("src").join("val.rs");

    let mut urls = RequestUrls::new();
    urls.append("VERIFY_IMAGE_URI", "tool/verify-image");
    urls.append("USER_LOGIN_URI", "user/login");
    urls.append("USER_REGISTER_URI", "user/register");
    urls.append("USER_LOGOUT_URI", "user/logout");
    urls.append("USER_INFO_URI", "user/info");
    urls.append("MANAGEMENT_LOGIN_URI", "management/login");
    urls.append("SITE_DATA_URI", "management/site_data");
    urls.append("BLOG_LIST_URI", "post/list/");
    urls.append("TAG_LIST_URI", "tag/list");
    urls.append("TOP_TAG_URI", "tag/top");
    urls.append("BLOG_TAG_LIST_URI", "post/tag/");
    urls.append("BLOG_SAVE_URI", "post/save");
    urls.append("BLOG_SHOW_URI", "post/show/");
    urls.append("BLOG_IMAGE_SAVE_URI", "post/image/save/");

    fs::write(&dest_path, &urls.0).unwrap();
}
