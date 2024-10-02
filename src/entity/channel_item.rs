//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "channel_item")]
pub struct Model {
    pub ordering: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub channel_id: i32,
    pub title: Option<String>,
    pub link: Option<String>,
    pub source: Option<String>,
    #[sea_orm(primary_key, auto_increment = false, unique)]
    pub enclosure: String,
    pub description: Option<String>,
    pub guid: Option<String>,
    pub pub_date: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::channel::Entity",
        from = "Column::ChannelId",
        to = "super::channel::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Channel,
    #[sea_orm(has_many = "super::listening_state::Entity")]
    ListeningState,
}

impl Related<super::channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channel.def()
    }
}

impl Related<super::listening_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ListeningState.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
