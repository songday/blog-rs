use std::io::Write;

use lazy_static::lazy_static;
use tera::Tera;
use zip::write::FileOptions;

use crate::db::model::Post;
use crate::db::post;
use crate::util::{self, result::Result};

static HUGO_TEMPLATE: &'static str = include_str!("../resource/static-site/template/hugo.txt");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        if let Err(e) = tera.add_raw_template("hugo.md", HUGO_TEMPLATE) {
            eprintln!("{:?}", e);
        }
        tera
    };
}

fn render(post: &Post, template: &str) -> String {
    let mut context = tera::Context::new();
    context.insert("title", &post.title);
    context.insert("content", &post.markdown_content);
    match TEMPLATES.render(template, &context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            String::new()
        },
    }
}

pub async fn hugo() -> Result<String> {
    let posts = post::all().await?;

    let export_dir = std::env::current_dir()?.join("export");
    if !export_dir.exists() {
        tokio::fs::create_dir(export_dir.as_path()).await?;
    }
    let mut filename = util::common::simple_uuid();
    filename.push_str(".zip");
    let output_file = export_dir.join(filename.as_str());
    let file = std::fs::File::create(output_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let mut file_name = String::with_capacity(32);
    for post in posts.iter() {
        file_name.push_str(post.id.to_string().as_str());
        file_name.push_str(".md");
        zip.start_file(file_name.as_str(), FileOptions::default())?;

        let content = render(post, "hugo.md");
        zip.write_all(content.as_bytes())?;

        file_name.clear();
    }
    zip.finish()?;

    Ok(filename)
}
