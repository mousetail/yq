use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

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
}

impl Solution {
    pub async fn get_solutions_for_challenge_and_language(
        pool: &Pool<Postgres>,
        challenge_id: i32,
        language: &str,
    ) -> Vec<Self> {
        let sql = "SELECT id, language, version, challenge, code FROM solutions WHERE challenge=$1 AND language=$2";
        sqlx::query_as::<_, Solution>(sql)
            .bind(challenge_id)
            .bind(language)
            .fetch_all(pool)
            .await
            .unwrap()
    }
}
