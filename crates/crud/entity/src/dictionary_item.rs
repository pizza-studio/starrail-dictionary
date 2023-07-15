use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use model::Language;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "dictionary_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub vocabulary_id: i32,
    pub language: Language,
    pub vocabulary_translation: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::VocabularyId", to = "Column::VocabularyId")]
    SelfReferencing
}

pub struct SelfReferencingLink;

impl Linked for SelfReferencingLink {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::SelfReferencing.def()
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}
