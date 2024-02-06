use migration::{DbErr, Migrator};
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Statement,
};
use sea_orm_migration::MigratorTrait;
use sea_query::{self, Query};

use tracing_unwrap::{self, ResultExt};

use std::sync::Arc;

use tracing::{info, warn};

use model::NestedDictionaryItem;

use anyhow::Result;

pub use entity::dictionary_item;
pub use sea_orm::ActiveValue::{NotSet, Set, Unchanged};
pub use sea_orm::DbConn;

pub async fn insert_item(
    new_items: Vec<dictionary_item::ActiveModel>,
    db: &DbConn,
) -> Result<usize> {
    let count = new_items.len();

    for items in new_items.chunks(800) {
        dictionary_item::Entity::insert_many(items.to_vec())
            .exec(db)
            .await?;
    }

    info!("{} new dictionary items inserted", count);
    Ok(count)
}

pub async fn search_dictionary_items(
    search_word: &str,
    page_size: u64,
    page: Option<u64>,
    db: Arc<DbConn>,
) -> Result<(u64, Vec<NestedDictionaryItem>)> {
    let sql = dictionary_item::Entity::find()
        .filter(
            dictionary_item::Column::Id.in_subquery(
                Query::select()
                    .column(dictionary_item::Column::Id)
                    .from(dictionary_item::Entity)
                    .and_where(dictionary_item::Column::VocabularyTranslation.contains(search_word))
                    .distinct_on([dictionary_item::Column::VocabularyId])
                    .to_owned(),
            ),
        )
        .order_by_asc(sea_query::Expr::cust_with_expr(
            "LENGTH($1)",
            sea_query::Expr::col((
                dictionary_item::Entity,
                dictionary_item::Column::VocabularyTranslation,
            )),
        ));

    let total_page = {
        let total_page = sql.clone().count(&*db.clone()).await?;
        if total_page == 0 {
            0
        } else {
            ((total_page as f64) / (page_size as f64)).ceil() as u64
        }
    };

    let handlers = sql
        .offset((page.unwrap_or(1) - 1) * page_size)
        .limit(page_size)
        .all(&*db.clone())
        .await?
        .into_iter()
        .map(|item| {
            let db = db.clone();
            tokio::spawn(async move {
                dictionary_item::Entity::find()
                    .filter(dictionary_item::Column::VocabularyId.eq(item.vocabulary_id))
                    .all(&*db)
                    .await
                    .map(|results| (item, results))
            })
        })
        .collect::<Vec<_>>();

    let mut results = vec![];

    for handle in handlers.into_iter() {
        let (target, items) = handle.await??;
        results.push(NestedDictionaryItem {
            vocabulary_id: target.vocabulary_id,
            target: target.vocabulary_translation,
            target_lang: target.language,
            lan_dict: items
                .into_iter()
                .map(|item| (item.language, item.vocabulary_translation))
                .collect(),
        })
    }

    Ok((total_page, results))
}

pub async fn establish_connection() -> Result<DbConn, DbErr> {
    info!("Initializing database connection...");

    let db_user = std::env::var("DATABASE_USER")
        .expect("Unable to find DATABASE_USER in environment variables");
    let db_password = std::env::var("DATABASE_PASSWORD")
        .expect("Unable to find DATABASE_PASSWORD in environment variables");

    const DB_NAME: &str = "starrail_dictionary";

    info!(
        "
        Connecting to database.
        DB_USER: {db_user}
        DB_NAME: {DB_NAME}
        host: db:5432
    "
    );

    let database_url = format!("postgresql://{db_user}:{db_password}@db:5432/{DB_NAME}");

    let conn_result = Database::connect(&database_url).await;

    let db = match conn_result {
        Ok(db) => db,
        Err(err) => {
            warn!(?err);
            let init_db_url = format!("postgresql://{db_user}:{db_password}@db:5432/postgres");
            let db = Database::connect(&init_db_url).await.unwrap_or_log();
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE {DB_NAME}"),
            ))
            .await
            .unwrap_or_log();
            Database::connect(&database_url).await.unwrap_or_log()
        }
    };

    info!("Database connection init succeeded. ");

    info!("Applying migration...");
    Migrator::up(&db, None).await.unwrap_or_log();
    info!("Migration succeeded");

    Ok(db)
}

pub async fn delete_all_dictionary(db: &DbConn) -> Result<(), DbErr> {
    info!("Deleting all items");

    dictionary_item::Entity::delete_many()
        .filter(sea_query::Expr::cust("1 = 1"))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete_duplicate_items(db: &DbConn) -> Result<(), DbErr> {
    info!("Deleting duplicated items");
    let exec_res = db
        .execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "
            DELETE FROM dictionary_item
                WHERE
                    vocabulary_id NOT IN (
                        SELECT MIN(vocabulary_id)
                        FROM (
                            SELECT
                                vocabulary_id,
                                STRING_AGG(vocabulary_translation, ', ') AS translations
                            FROM (
                                SELECT
                                    vocabulary_id,
                                    vocabulary_translation,
                                    language
                                FROM
                                    dictionary_item
                                ORDER BY
                                    language
                            ) AS sorted_items
                            GROUP BY
                                vocabulary_id
                        ) AS subquery_alias
                        GROUP BY translations
                    );
            "
            .to_string(),
        ))
        .await?;

    info!(
        "{} duplicated items was deleted. ",
        exec_res.rows_affected()
    );
    Ok(())
}
