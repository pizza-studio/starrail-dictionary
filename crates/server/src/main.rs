use std::{net::SocketAddr, sync::Arc, vec};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use clap::Parser;
use tower_http::cors::CorsLayer;

use axum_valid::Valid;

use model::{SearchApiResult, SearchParams};

use crud::{establish_connection, search_dictionary_items, DbConn};

use tracing::{error, info, warn, Level};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer, Registry,
};
use tracing_unwrap::ResultExt;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    update: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _guards = init_tracing();
    info!("Args: {args:?}");

    info!("Establishing database connection");

    warn!("test warning");
    error!("test error");


    let db = establish_connection().await.unwrap_or_log();
    let shared_state = Arc::new(db);

    if args.update {
        info!("Updating dictionary data");
        update_data::update_all_data(shared_state.clone())
            .await
            .unwrap_or_log();
    }

    info!("Starting server...");

    let app = Router::new()
        .route("/:version/translations/:query", get(search_dictionary))
        .with_state(shared_state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap_or_log();
}

#[axum_macros::debug_handler]
async fn search_dictionary(
    Valid(Query(SearchParams { page_size, page })): Valid<Query<SearchParams>>,
    Path((version, query)): Path<(String, String)>,
    State(db): State<Arc<DbConn>>,
) -> Result<Json<SearchApiResult>, (StatusCode, &'static str)> {
    info!(?page_size, ?page, "Searching '{}'. ", &query);

    match &*version {
        "v1" => {
            let (total_page, translations) = (!query.is_empty())
                .then_some(search_dictionary_items(&query, page_size, page, db).await)
                .unwrap_or(Ok((0, vec![])))
                .map_err(|err| {
                    error!("{:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error. ")
                })?;

            Ok(Json(SearchApiResult {
                total: total_page,
                page: page.unwrap_or(0),
                page_size: page_size,
                translations: translations.into_iter().collect(),
            }))
        }
        _ => Err((StatusCode::NOT_FOUND, "API version invalid. ")),
    }
}

fn init_tracing() -> (non_blocking::WorkerGuard, non_blocking::WorkerGuard) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .pretty()
        .with_file(false)
        .with_writer(std::io::stderr);

    let file_appender = rolling::hourly("./logs/info", "hsrdict-info.log");
    let (non_blocking_appender, guard1) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .pretty()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    let error_file_appender = rolling::daily("./logs/warn", "hsrdict-warn.log");
    let (error_non_blocking_appender, guard2) = non_blocking(error_file_appender);
    let error_file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(error_non_blocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::WARN));

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(error_file_layer)
        .with(file_layer)
        .init();

    (guard1, guard2)
}
