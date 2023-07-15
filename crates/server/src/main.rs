use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Query, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use model::{NestedDictionaryItem, SearchParams};

use crud::{establish_connection, search_dictionary_items, DbConn};

use tracing::{debug, error, info};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};
use tracing_unwrap::ResultExt;

use axum_macros;

use serde::{Deserialize, Serialize};
use serde_json;

struct AppState {
    db: DbConn,
}

#[tokio::main]
async fn main() {
    init_tracing();

    info!("Starting server...");

    let shared_state = Arc::new(AppState {
        db: establish_connection().await.unwrap_or_log(),
    });

    let app = Router::new()
        .route("/search", get(search_dictionary))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn search_dictionary(
    Query(SearchParams {
        search_word,
        batch_size,
        page,
    }): Query<SearchParams>,
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<NestedDictionaryItem>>, (StatusCode, &'static str)> {
    info!(?batch_size, ?page, "Searching '{}'. ", &search_word);
    if search_word.is_empty() {
        info!("Searching word is empty. Return nothing. ");
        return Ok(Json(vec![]));
    }
    search_dictionary_items(&search_word, batch_size, page, &app_state.db)
        .await
        .map_err(|err| {
            error!("Error in search_dictionary: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })
        .map(|data| Json(data))
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .pretty()
        .with_file(false)
        .with_writer(std::io::stderr);

    let file_appender = rolling::hourly("./logs/backend", "app.log");
    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
        .init();
}
