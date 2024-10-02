use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewSolution {
    pub language: String,
    pub version: String,
    pub challenge: i32,
    pub code: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Solution {
    pub id: i32,
    #[sqlx(flatten)]
    pub solution: NewSolution,
}
