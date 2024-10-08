use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewSolution {
    pub code: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Solution {
    pub id: i32,
    pub language: String,
    pub version: String,
    pub challenge: i32,
    #[sqlx(flatten)]
    pub solution: NewSolution,
    pub author: i32,
    pub score: i32,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub author_id: i32,
    pub author_name: String,
    pub score: i32,
}

impl LeaderboardEntry {
    pub async fn get_leadeboard_for_challenge_and_language(
        pool: &PgPool,
        challenge_id: i32,
        language: &str,
    ) -> Vec<Self> {
        sqlx::query_as!(
            LeaderboardEntry,
            "
            SELECT solutions.id as id, solutions.author as author_id, accounts.username as author_name, score FROM solutions
            LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2
            ",
            challenge_id,
            language
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }
}
