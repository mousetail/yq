mod cachemap;
mod error;
mod langs;
mod run;

use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use cachemap::CacheMap;
use error::RunLangError;
use run::{get_lang_versions, process_message};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub enum Message {
    Install {
        lang: String,
        version: String,
    },
    Run {
        lang: String,
        version: String,
        code: String,
    },
}

// fn main() {
//     println!("Starting!");

//     let mut lang_versions = get_lang_versions();
//     println!("{lang_versions:?}");

//     let messages = [
//         Message::Install {
//             lang: "nodejs".to_owned(),
//             version: "17.3.0".to_owned(),
//         },
//         Message::Install {
//             lang: "python".to_owned(),
//             version: "3.12.0".to_owned(),
//         },
//         Message::Run {
//             lang: "nodejs".to_owned(),
//             version: "17.3.0".to_owned(),
//             code: "console.log(\"Hello World!\");".to_owned(),
//         },
//         Message::Run {
//             lang: "python".to_owned(),
//             version: "3.12.0".to_owned(),
//             code: "import math\nprint(f\"Hello World! {math.sqrt(25)}\");".to_owned(),
//         },
//     ];

//     for message in messages {
//         println!("processing message {message:?}");
//         process_message(message, &mut lang_versions).unwrap();
//     }

//     let lang_versions = get_lang_versions();
//     println!("{lang_versions:?}");
// }

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let lang_versions = get_lang_versions().await;

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/", post(handle_message))
        .with_state(Arc::new(lang_versions));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    return "Server is working properly";
}

#[axum::debug_handler]
async fn handle_message(
    lang_versions: State<Arc<CacheMap<String, CacheMap<String, ()>>>>,
    message: Json<Message>,
) -> Result<&'static str, RunLangError> {
    process_message(message.0, &lang_versions.0).await?;
    return Ok("");
}
