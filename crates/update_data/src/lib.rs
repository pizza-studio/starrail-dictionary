use std::{collections::HashMap, rc::Rc, sync::Arc};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::{self, task::JoinSet};

use model::Language;

use crud::{dictionary_item, establish_connection, insert_item, NotSet, Set, Unchanged, delete_all_dictionary, delete_duplicate_items};

use anyhow::{self, Ok};

use sea_orm::Iterable;
use strum::IntoEnumIterator;

use tracing::{debug, error, info};

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
        .into_iter()
        .collect()
    };
}

pub async fn update_all_data() -> anyhow::Result<()> {
    let db = Arc::new(establish_connection().await?);

    // let mut set: JoinSet<anyhow::Result<()>> = JoinSet::new();

    // delete_all_dictionary(&db).await?;

    // LANGUAGE_URL_MAPPING.iter().for_each(|(lang, url)| {
    //     let db = db.clone();
    //     set.spawn(async move {
    //         info!("Getting data for {}", lang);
    //         let dictionary_map = reqwest::get(url)
    //             .await?
    //             .json::<HashMap<i32, String>>()
    //             .await?;
    //         info!("Updating data for {}", lang);
    //         insert_item(
    //             dictionary_map
    //                 .into_iter()
    //                 .map(|(word_id, word_translation)| dictionary_item::ActiveModel {
    //                     vocabulary_id: Set(word_id),
    //                     language: Set(lang.clone()),
    //                     vocabulary_translation: Set(word_translation),
    //                     ..Default::default()
    //                 })
    //                 .collect(),
    //             &db,
    //         )
    //         .await?;
    //         info!("Data for {} updated", lang);
    //         Ok(())
    //     });
    // });

    // while let Some(handle) = set.join_next().await {
    //     handle??;
    // }

    delete_duplicate_items(&db).await?;

    Ok(())
}
