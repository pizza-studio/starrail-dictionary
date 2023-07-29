use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DictionaryItem::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DictionaryItem::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DictionaryItem::VocabularyId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DictionaryItem::Language).string().not_null())
                    .col(
                        ColumnDef::new(DictionaryItem::VocabularyTranslation)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-vocabulary_id")
                    .table(DictionaryItem::Table)
                    .col(DictionaryItem::VocabularyId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-vocabulary_translation").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-vocabulary_id").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DictionaryItem::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum DictionaryItem {
    Table,
    Id,
    VocabularyId,
    Language,
    VocabularyTranslation,
}
