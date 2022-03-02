use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GitRepositoryInfo {
    pub name: String,
    pub email: String,
    pub remote_url: String,
    pub repository_name: String,
    pub last_export_second: i64,
}
