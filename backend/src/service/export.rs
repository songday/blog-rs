use std::io::Write;

use zip::write::FileOptions;

use crate::db::model::Post;
use crate::db::post;
use crate::util::{self, result::Result};

static HUGO_TEMPLATE: &'static str = include_str!("../resource/static-site/template/hugo.txt");

fn render(post: &Post, template: &'static str) -> String {
    let mut context = tera::Context::new();
    context.insert("title", &post.title);
    context.insert("content", &post.markdown_content);
    let mut tera = tera::Tera::default();
    match tera.render_str(template, &context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            String::new()
        },
    }
}

pub async fn hugo() -> Result<String> {
    let posts = post::all().await?;

    let mut filename = util::common::simple_uuid();
    filename.push_str(".zip");
    let export_dir = std::env::current_dir()?.join("export");
    if !export_dir.exists() {
        std::fs::create_dir(export_dir.as_path())?;
    }
    let output_file = export_dir.join(filename.as_str());
    let file = std::fs::File::create(output_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let mut file_name = String::with_capacity(32);
    for post in posts.iter() {
        file_name.push_str(post.id.to_string().as_str());
        file_name.push_str(".md");
        zip.start_file(file_name.as_str(), FileOptions::default())?;

        let content = render(post, HUGO_TEMPLATE);
        zip.write_all(content.as_bytes())?;

        file_name.clear();
    }
    zip.finish()?;

    Ok(filename)
}
