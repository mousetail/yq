use common::RunLangOutput;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::Error;

#[derive(sqlx::FromRow, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct NewChallenge {
    pub description: String,
    pub judge: String,
    pub name: String,
    pub example_code: String,
}

impl Default for NewChallenge {
    fn default() -> Self {
        NewChallenge {
            description: concat!(
                "Explain in detail how to solve your challenge. Good challenge descriptions ",
                "include examples and links to relevent resources. Markdown is supported"
            )
            .to_string(),
            judge: concat!(
                "(async function*(context: Context): Challenge {\n",
                "  yield (await context.run(undefined)).assertEquals('Hello World!');\n",
                "  //Your code here\n",
                "  return context.noFailures();\n",
                "})"
            )
            .to_string(),
            name: String::new(),
            example_code: String::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum NewOrExistingChallenge {
    Existing(ChallengeWithAuthorInfo),
    New(NewChallenge),
}
impl NewOrExistingChallenge {
    pub fn get_new_challenge(&self) -> &NewChallenge {
        match self {
            Self::Existing(e) => &e.challenge.challenge,
            Self::New(k) => k,
        }
    }

    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, Error> {
        Ok(ChallengeWithAuthorInfo::get_by_id(pool, id)
            .await?
            .map(NewOrExistingChallenge::Existing))
    }
}

impl Default for NewOrExistingChallenge {
    fn default() -> Self {
        Self::New(NewChallenge::default())
    }
}

#[derive(Serialize)]
pub struct ChallengeWithTests {
    #[serde(flatten)]
    pub challenge: NewOrExistingChallenge,
    pub tests: RunLangOutput,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct Challenge {
    pub id: Option<i32>,
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub challenge: NewChallenge,
    pub author: i32,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct ChallengeWithAuthorInfo {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub challenge: Challenge,
    pub author_name: String,
    pub author_avatar: String,
}

impl ChallengeWithAuthorInfo {
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, Error> {
        let sql = "SELECT
            challenges.id,
            challenges.name,
            challenges.description,
            challenges.judge,
            challenges.example_code,
            challenges.author,
            accounts.username as author_name,
            accounts.avatar as author_avatar
            FROM challenges LEFT JOIN accounts ON challenges.author = accounts.id
            WHERE challenges.id=$1
            "
        .to_string();

        let challenge: Option<ChallengeWithAuthorInfo> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(Error::DatabaseError)?;

        Ok(challenge)
    }
}
