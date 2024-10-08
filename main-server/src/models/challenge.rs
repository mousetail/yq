use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::Error;

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

impl Challenge {
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Challenge, Error> {
        let sql = "SELECT * FROM challenges WHERE id=$1".to_string();

        let challenge: Challenge = sqlx::query_as(&sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|_| Error::NotFound)?;

        Ok(challenge)
    }
}
