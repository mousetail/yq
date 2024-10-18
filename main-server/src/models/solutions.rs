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

#[derive(Serialize)]
pub struct Code {
    pub code: String,
    pub score: i32,
    pub id: i32,
}

impl Code {
    pub async fn get_best_code_for_user(
        pool: &PgPool,
        account: i32,
        challenge: i32,
        language: &str,
    ) -> Option<Code> {
        sqlx::query_as!(
            Code,
            r#"
                SELECT code, score, id from solutions
                WHERE author=$1 AND challenge=$2 AND language=$3
                ORDER BY score ASC
                LIMIT 1
            "#,
            account,
            challenge,
            language
        )
        .fetch_optional(pool)
        .await
        .expect("Database connection error")
    }
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub author_id: i32,
    pub author_name: String,
    pub author_avatar: String,
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
            SELECT
                solutions.id as id,
                solutions.author as author_id,
                accounts.username as author_name,
                accounts.avatar as author_avatar,
                score FROM solutions
            LEFT JOIN accounts ON solutions.author = accounts.id
            WHERE solutions.challenge=$1 AND solutions.language=$2
            ORDER BY solutions.score ASC
            ",
            challenge_id,
            language
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }
}
