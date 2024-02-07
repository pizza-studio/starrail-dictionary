use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;



use model::Language;

use crud::{
    delete_all_dictionary, delete_duplicate_items, dictionary_item,
    insert_item, Set,
};

use anyhow::{self, Ok};

use sea_orm::{Iterable, DbConn};

use tracing::info;

lazy_static! {
    static ref LANGUAGE_URL_MAPPING: HashMap<Language, String> = {
        Language::iter()
        .map(|lang| {
            let url = format!(
                "https://raw.githubusercontent.com/CanglongCl/StarRailData/master/TextMap/TextMap{}.json",
                lang.str_id().to_uppercase()
            );
            info!("Data url for {} is: {}", lang.str_id(), url);
            (
                lang.clone(),
                url,
            )
        })
        .collect()
    };
}

pub async fn update_all_data(db: Arc<DbConn>) -> anyhow::Result<()> {
    delete_all_dictionary(&db).await?;

    for (lang, url) in LANGUAGE_URL_MAPPING.iter() {
        info!("Getting data for {}", lang.str_id());
        let dictionary_map = reqwest::get(url)
            .await?
            .json::<HashMap<i32, String>>()
            .await?;
        info!("Updating data for {}", lang.str_id());
        let item_inserted_count = insert_item(
            dictionary_map
                .into_iter()
                .map(|(word_id, word_translation)| dictionary_item::ActiveModel {
                    vocabulary_id: Set(word_id),
                    language: Set(lang.clone()),
                    vocabulary_translation: Set(word_translation),
                    ..Default::default()
                })
                .collect(),
            &db,
        )
        .await?;
        info!("Data for {} updated ({})", lang.str_id(), item_inserted_count);
    }

    delete_duplicate_items(&db).await?;

    Ok(())
}
