use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                        .col(ColumnDef::new(Channel::Id).integer().not_null().auto_increment().primary_key())
                        .col(ColumnDef::new(Channel::Title).string())
                        .col(ColumnDef::new(Channel::Link).unique_key().string())
                        .col(ColumnDef::new(Channel::Description).string())
                        // .foreign_key(
                        //     ForeignKey::create()
                        //         .name("id_fk")
                        //         .from(Channel::Table, Channel::Id)
                        //         .to(ChannelItem::Table, ChannelItem::ChannelId)
                        //         .on_delete(ForeignKeyAction::Cascade)
                        //     )
                        .to_owned()
                )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChannelItem::Table)
                    .if_not_exists()
                        .col(ColumnDef::new(ChannelItem::Id).integer().not_null().auto_increment().primary_key())
                        .col(ColumnDef::new(ChannelItem::ChannelId).integer().not_null())
                        .col(ColumnDef::new(ChannelItem::Title).string())
                        .col(ColumnDef::new(ChannelItem::Link).string())
                        .col(ColumnDef::new(ChannelItem::Description).string())
                        .col(ColumnDef::new(ChannelItem::Guid).uuid())
                        .col(ColumnDef::new(ChannelItem::PubDate).date_time())
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_channel")
                                .from(ChannelItem::Table, ChannelItem::ChannelId)
                                .to(Channel::Table, Channel::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                            )
                        .to_owned()
                )
            .await?;

        self.seed_data(&manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ChannelItem::Table).to_owned())
            .await?;

        Ok(())
    }
}

impl Migration {
    async fn seed_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let q1 = Query::insert().into_table(Channel::Table)
            .columns([Channel::Title, Channel::Link, Channel::Description])
            .values_panic(["Dasko i Mladja".into(), "http://daskoimladha.com".into(), "desc".into()])
            .values_panic(["Agelast".into(), "http://agelast.com".into(), "desc2".into()])
            .to_owned();

        manager.exec_stmt(q1).await?;

        let q2 = Query::insert().into_table(ChannelItem::Table)
            .columns([ChannelItem::ChannelId, ChannelItem::Title, ChannelItem::Link, ChannelItem::Description])
            .values_panic([1.into(), "D i M1".into(), "http://link1.com".into(), "desc".into()])
            .values_panic([1.into(), "D i M2".into(), "http://link2.com".into(), "desc".into()])
            .values_panic([1.into(), "D i M3".into(), "http://link3.com".into(), "desc".into()])
            .values_panic([2.into(), "agelas1".into(), "http://link4.com".into(), "desc".into()])
            .values_panic([2.into(), "agelas2".into(), "http://link5.com".into(), "desc".into()])
            .values_panic([2.into(), "agelas3".into(), "http://link5.com".into(), "desc".into()])
            .to_owned();

        manager.exec_stmt(q2).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Id,
    Title,
    Link,
    Description
}

#[derive(DeriveIden)]
enum ChannelItem {
    Table,
    Id,
    ChannelId,
    Title,
    Link,
    Description,
    Guid,
    PubDate
}

