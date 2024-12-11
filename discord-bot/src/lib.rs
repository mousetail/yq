
use serenity::all::{
        ChannelId, CreateEmbed, CreateMessage, EditMessage,
        MessageId,
    };
use sqlx::PgPool;
use tokio::sync::mpsc::{Receiver, Sender};

// impl Bot {
//     fn notify_best_score_changed(&self) {
//         self.client.
//     }
// }

pub struct ScoreImproved {
    pub challenge_id: i32,
    pub author: i32,
    pub language: String,
    pub score: i32,
}

struct LastMessage {
    id: i32,
    language: String,
    challenge_id: i32,
    author_id: i32,
    author_name: String,
    score: i32,
    previous_author_id: Option<i32>,
    previous_author_name: Option<String>,
    previous_author_score: Option<i32>,
    message_id: i64,
    channel_id: i64,
}

fn format_message(
    previous_message: &Option<LastMessage>,
    new_message: &ScoreImproved,
    challenge_name: &str,
    author_name: &str,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("Improved score for {challenge_name}"))
        .url(format!(
            "{}/challenge/{}/{}/solve/{}",
            std::env::var("YQ_PUBLIC_URL").unwrap(),
            new_message.challenge_id,
            slug::slugify(challenge_name),
            new_message.language
        ));
    if let Some(previous) = previous_message {
        let (previous_author, previous_score) = if previous.author_id == new_message.author {
            if let (Some(previous_author_name), Some(previous_author_score)) = (
                &previous.previous_author_name,
                &previous.previous_author_score,
            ) {
                (previous_author_name.clone(), previous_author_score.clone())
            } else {
                (previous.author_name.clone(), previous.score)
            }
        } else {
            (previous.author_name.clone(), previous.score)
        };
        embed = embed.field(previous_author, format!("{}", previous_score), false);
    }
    embed = embed.field(author_name, format!("{}", new_message.score), false);

    embed
}

async fn save_new_message_info(
    pool: &PgPool,
    last_message: Option<LastMessage>,
    message: ScoreImproved,
    message_id: i64,
    last_author_id: Option<i32>,
    last_score: Option<i32>,
    final_channel_id: i64,
) -> Result<(), sqlx::Error> {
    match &last_message {
        Some(e) => {
            sqlx::query!(
                "UPDATE discord_messages
                SET author=$1,
                score=$2,
                previous_author=$3,
                previous_author_score=$4,
                message_id=$5,
                channel_id=$6
                WHERE id=$7",
                message.author,
                message.score,
                last_author_id,
                last_score,
                message_id,
                final_channel_id,
                e.id
            )
            .execute(pool)
            .await
        }
        None => {
            sqlx::query!(
                r#"INSERT INTO discord_messages
                (
                    language,
                    challenge,
                    author,
                    previous_author,
                    previous_author_score,
                    score,
                    message_id,
                    channel_id
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8 
                )"#,
                message.language,
                message.challenge_id,
                message.author,
                last_author_id,
                last_score,
                message.score,
                message_id,
                final_channel_id
            )
            .execute(pool)
            .await
        }
    }?;
    Ok(())
}

async fn get_last_message(
    pool: &PgPool,
    challenge: i32,
    language: &str,
) -> Result<Option<LastMessage>, sqlx::Error> {
    sqlx::query_as!(
        LastMessage,
        r#"
        SELECT discord_messages.id,
            discord_messages.language,
            discord_messages.author as author_id,
            discord_messages.challenge as challenge_id,
            accounts.username as author_name,
            discord_messages.previous_author as previous_author_id,
            discord_messages.score as score,
            previous_account.username as "previous_author_name?",
            discord_messages.previous_author_score,
            discord_messages.message_id,
            discord_messages.channel_id
        FROM discord_messages
        INNER JOIN accounts ON discord_messages.author = accounts.id
        LEFT JOIN accounts as previous_account ON discord_messages.previous_author = previous_account.id
        WHERE discord_messages.language=$1 AND discord_messages.challenge=$2
        "#,
        language,
        challenge,
    ).fetch_optional(pool).await
}

async fn handle_bot_queue(
    mut receiver: Receiver<ScoreImproved>,
    http_client: serenity::http::Http,
    pool: PgPool,
    channel_id: ChannelId,
) {
    match http_client.get_current_application_info().await {
        Ok(o) => {
            println!("Discord Bot Initialized, user name: {:?}", o.name)
        }
        Err(e) => {
            eprint!("Failed to initalize disocrd bot: {:?}", e);
            return;
        }
    }

    while let Some(message) = receiver.recv().await {
        let last_message =
            match get_last_message(&pool, message.challenge_id, &message.language).await {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Faied to fetch previous message: {err:?}");
                    continue;
                }
            };

        let challenge_name = match sqlx::query_scalar!(
            "
            SELECT name
            FROM challenges
            WHERE id=$1
            ",
            message.challenge_id
        )
        .fetch_one(&pool)
        .await
        {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Failed to fetch challenge id: {err:?}");
                continue;
            }
        };
        let author_name = match sqlx::query_scalar!(
            "SELECT username
            FROM accounts
            WHERE id=$1
            ",
            message.author
        )
        .fetch_one(&pool)
        .await
        {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Failed to fetch challenge id: {err:?}");
                continue;
            }
        };

        let formatted_message =
            format_message(&last_message, &message, &challenge_name, &author_name);
        let (message_id, last_author_id, last_score, final_channel_id) = if let Some(k) =
            last_message
                .as_ref()
                .filter(|e| e.author_id == message.author)
        {
            match ChannelId::new(u64::from_be_bytes(k.channel_id.to_be_bytes()))
                .edit_message(
                    &http_client,
                    MessageId::new(u64::from_be_bytes(k.message_id.to_be_bytes())),
                    EditMessage::new().embed(formatted_message),
                )
                .await
            {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Failed to update message: {err:?}");
                    continue;
                }
            };
            (
                k.message_id,
                k.previous_author_id,
                k.previous_author_score,
                k.channel_id,
            )
        } else {
            let response = match channel_id
                .send_message(&http_client, CreateMessage::new().embed(formatted_message))
                .await
            {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Failed to update message: {err:?}");
                    continue;
                }
            };

            let (last_author_id, last_author_score) = match &last_message {
                Some(e) => (Some(e.author_id), Some(e.score)),
                None => (None, None),
            };
            (
                i64::from_be_bytes(response.id.get().to_be_bytes()),
                last_author_id,
                last_author_score,
                i64::from_be_bytes(channel_id.get().to_be_bytes()),
            )
        };

        if let Err(e) = save_new_message_info(
            &pool,
            last_message,
            message,
            message_id,
            last_author_id,
            last_score,
            final_channel_id,
        )
        .await
        {
            eprint!("Failed to update database entry {e:?}");
            continue;
        }
    }
}

pub fn init_bot(pool: PgPool, discord_token: String, channel_id: u64) -> Sender<ScoreImproved> {
    let (sender, receiver) = tokio::sync::mpsc::channel::<ScoreImproved>(32);
    let http_client = serenity::http::Http::new(&discord_token);

    let channel = ChannelId::new(channel_id);

    println!("init bot called");
    tokio::spawn(handle_bot_queue(receiver, http_client, pool, channel));

    return sender;
}

#[derive(Clone)]
pub struct Bot {
    pub channel: Option<Sender<ScoreImproved>>,
}

impl Bot {
    pub async fn send(&self, message: ScoreImproved) {
        if let Some(channel) = &self.channel {
            if let Err(e) = channel.send(message).await {
                eprintln!("Error sending: {e:?}",);
            }
        }
    }
}
