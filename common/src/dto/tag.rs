use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagUsageAmount {
    pub id: i64,
    pub name: String,
    pub amount: u32,
}
