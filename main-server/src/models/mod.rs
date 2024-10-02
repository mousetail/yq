use serde::{Deserialize, Serialize};

pub mod challenge;
pub mod solutions;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct InsertedId(pub i32);
