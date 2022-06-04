use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GitRepositoryInfo {
    pub name: String,
    pub email: String,
    pub remote_url: String,
    pub repository_name: String,
    pub branch_name: Option<String>,
    pub last_export_second: i64,
}

#[derive(Deserialize, Serialize)]
pub struct GitPushInfo {
    pub subdirectory: String,
    pub render_html: bool,
    pub repo_credential: String,
}
