use redis::{
    aio::{ConnectionManager, MultiplexedConnection},
    AsyncCommands,
};
use time::{Date, OffsetDateTime, Time};

pub struct LeaderboardUser {
    username: String,
    id: i32,
    avatar: String,
    score: i32,
    timestamp: OffsetDateTime,
}

// I can spend up to 52 bits total
const SCORE_MAX_BITS: usize = 12;
const TIMESTAMP_MAX_BITS: usize = 40;

impl LeaderboardUser {
    fn get_score_float(&self) -> f64 {
        if i32::BITS - self.score.leading_zeros() > SCORE_MAX_BITS as u32 {
            panic!("Score should be at most 2^{SCORE_MAX_BITS}")
        }

        let score = self.score as f64;
        let tiebreaker = self.timestamp.unix_timestamp()
            - OffsetDateTime::new_utc(
                Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
                Time::MIDNIGHT,
            )
            .unix_timestamp();

        if i64::BITS - tiebreaker.leading_zeros() > TIMESTAMP_MAX_BITS as u32 {
            panic!("Timestamp should be at most 2^{TIMESTAMP_MAX_BITS}")
        }
        return score + (tiebreaker as f64) * (-(TIMESTAMP_MAX_BITS as f64)).exp2();
    }
}

pub struct RankingService {
    connection: ConnectionManager,
}

struct ChallengeLeaderboard {
    challenge_id: i32,
    language: String,
}

impl ChallengeLeaderboard {
    fn get_name(&self) -> String {
        format!(
            "challenge_scores\x00{}\x00{}",
            self.challenge_id, self.language
        )
    }
}

impl RankingService {
    pub async fn new() -> RankingService {
        let client = redis::Client::open("redis://localhost:6666")
            .expect("Failed to connect to parse Redis URL");
        let manager = ConnectionManager::new(client)
            .await
            .expect("Failed to connect to KvRocks");

        return RankingService {
            connection: manager,
        };
    }

    pub async fn get_users_near(&mut self, user_id: i32, leaderboard: ChallengeLeaderboard) {
        let rank: i32 = self
            .connection
            .zrank(leaderboard.get_name(), user_id)
            .await
            .unwrap();
    }

    pub async fn insert_user(&mut self, user: LeaderboardUser, leaderboard: ChallengeLeaderboard) {
        let _: i32 = self
            .connection
            .zadd(leaderboard.get_name(), user.id, user.get_score_float())
            .await
            .unwrap();
    }
}
