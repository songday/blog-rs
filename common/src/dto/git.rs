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
