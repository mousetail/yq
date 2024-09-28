use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewChallenge {
    pub description: String,
    pub judge: String,
    pub name: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Challenge {
    pub id: i32,
    #[sqlx(flatten)]
    pub challenge: NewChallenge,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct InsertedId(pub i32);
