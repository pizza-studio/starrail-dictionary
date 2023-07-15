use std::collections::HashMap;

use dotenv::dotenv;

use migration::{DbErr, Migrator};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, CursorTrait, Database, DatabaseBackend,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use sea_orm_migration::MigratorTrait;
use sea_query::{self, Query, Table};

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

pub async fn delete_duplicate_items(db: &DbConn) -> Result<(), DbErr> {
    info!("Deleting duplicated items");
    // dictionary_item::Entity::delete_many().filter(
    //     dictionary_item::Column::VocabularyId.in_subquery(
    //         Query::select()
    //             .column(dictionary_item::Column::VocabularyId.min())
    //             .from_subquery(
    //                 (Query::select().columns(cols).from_subquery(
    //                     (Query::select()
    //                         .columns([
    //                             dictionary_item::Column::VocabularyId,
    //                             dictionary_item::Column::VocabularyTranslation,
    //                             dictionary_item::Column::Language,
    //                         ])
    //                         .from(dictionary_item::Entity)
    //                         .order_by(dictionary_item::Column::Language, sea_orm::Order::Asc)),
    //                     Alias::new("subsub"),
    //                 )),
    //                 Alias::new("sub"),
    //             )
    //             .group_by_col("translations")
    //             .to_owned(),
    //     ),
    // );
    let exec_res = db
        .execute(sea_orm::Statement::from_string(
            DatabaseBackend::Sqlite,
            "
            DELETE FROM dictionary_item
            WHERE
                vocabulary_id NOT IN (
                    SELECT MIN(vocabulary_id)
                    FROM (
                            SELECT
                                vocabulary_id,
                                GROUP_CONCAT(vocabulary_translation, ', ') AS translations
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
                        )
                    GROUP BY translations
                )
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
