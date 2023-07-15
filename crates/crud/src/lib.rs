use std::collections::HashMap;

use dotenv::dotenv;

use migration::{DbErr, Migrator};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, CursorTrait, Database, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use sea_orm_migration::MigratorTrait;
use sea_query::{self, Table};

use entity::{prelude::*, *};

use tokio::task::JoinSet;
use tracing_unwrap::{self, ResultExt};

use tracing::info;

use model::NestedDictionaryItem;

pub use entity::dictionary_item;
pub use sea_orm::ActiveValue::{NotSet, Set, Unchanged};
pub use sea_orm::DbConn;

pub async fn insert_item(
    new_items: Vec<dictionary_item::ActiveModel>,
    db: &DbConn,
) -> Result<(), DbErr> {
    let count = new_items.len();

    for items in new_items.chunks(800).into_iter() {
        dictionary_item::Entity::insert_many(items.to_vec())
            .exec(db)
            .await?;
    }

    info!("{} new dictionary items inserted", count);
    Ok(())
}

pub async fn search_dictionary_items(
    search_word: &str,
    batch_size: u64,
    page: Option<u64>,
    db: &DbConn,
) -> Result<Vec<NestedDictionaryItem>, DbErr> {
    let items = dictionary_item::Entity::find()
        .filter(dictionary_item::Column::VocabularyTranslation.contains(search_word))
        .distinct()
        .find_with_linked(dictionary_item::SelfReferencingLink)
        .order_by_asc(sea_query::Expr::cust_with_expr(
            "LENGTH(?)",
            sea_query::Expr::col((
                dictionary_item::Entity,
                dictionary_item::Column::VocabularyTranslation,
            )),
        ))
        .offset(page.unwrap_or(0) * batch_size * 13)
        .limit(batch_size * 13) // Each word got 13 row
        .all(db)
        .await?
        .into_iter()
        .map(|(base, tranlations)| NestedDictionaryItem {
            vocabulary_id: base.vocabulary_id,
            target: base.vocabulary_translation,
            target_lang: base.language,
            lan_dict: tranlations
                .into_iter()
                .map(|translation| (translation.language, translation.vocabulary_translation))
                .collect(),
        })
        .collect::<Vec<_>>();
    Ok(items)
}

pub async fn establish_connection() -> Result<DbConn, DbErr> {
    info!("Initializing database connection...");
    dotenv().unwrap_or_log();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_log();
    info!("Database url: {}", database_url);
    let db = Database::connect(&database_url).await.unwrap_or_log();
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

pub async fn delete_duplicate_items(db: &DbConn) -> Result<Vec<dictionary_item::Model>, DbErr> {
    info!("Deleting duplicated items");
    let statement = dictionary_item::Entity::find()
        .filter(
            sea_query::Condition::all().add(
                sea_query::Expr::tuple([
                    sea_query::Expr::col(dictionary_item::Column::VocabularyTranslation).into(),
                    sea_query::Expr::col(dictionary_item::Column::Language).into(),
                ])
                .in_subquery(
                    sea_query::Query::select()
                        .columns([
                            dictionary_item::Column::VocabularyTranslation,
                            dictionary_item::Column::Language,
                        ])
                        .from(dictionary_item::Entity)
                        .group_by_columns([
                            dictionary_item::Column::VocabularyTranslation,
                            dictionary_item::Column::Language,
                        ])
                        .and_having(
                            sea_query::Expr::expr(
                                sea_query::Expr::col(
                                    dictionary_item::Column::VocabularyTranslation,
                                )
                                .count(),
                            )
                            .gt(1),
                        )
                        .to_owned(),
                ),
            ),
        )
        .build(sea_orm::DatabaseBackend::Sqlite);
        println!("{}", statement);
        todo!()
        // .all(db)
        // .await
}
