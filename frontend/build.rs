use std::{fs, path::Path};

struct RequestUrls(String);

impl RequestUrls {
    fn new() -> Self { RequestUrls(String::with_capacity(2048)) }
    fn append(&mut self, var_name: &str, uri: &str) {
        self.0.push_str("pub(crate) const ");
        self.0.push_str(var_name);
        self.0.push_str(": &str = \"");
        self.0.push_str("http://127.0.0.1:9270/");
        // self.0.push_str("https://www.songday.com/");
        self.0.push_str(uri);
        self.0.push_str("\";\n");
    }
}

fn main() {
    let dest_path = Path::new("src").join("val.rs");

    let mut urls = RequestUrls::new();
    urls.append("VERIFY_IMAGE_URL", "tool/verify-image");
    urls.append("USER_LOGIN_URL", "user/login");
    urls.append("USER_REGISTER_URL", "user/register");
    urls.append("USER_LOGOUT_URL", "user/logout");
    urls.append("USER_INFO_URL", "user/info");
    urls.append("BLOG_LIST_URL", "blog/list/");
    urls.append("BLOG_TAGS_URL", "blog/tags");
    urls.append("BLOG_TAG_LIST_URL", "blog/tag/");
    urls.append("BLOG_SAVE_URL", "blog/save");
    urls.append("BLOG_SHOW_URL", "blog/show/");
    urls.append("BLOG_IMAGE_SAVE_URL", "blog/image/save/");

    fs::write(&dest_path, &urls.0).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
