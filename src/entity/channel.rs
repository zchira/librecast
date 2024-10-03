//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "channel")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: Option<String>,
    #[sea_orm(unique)]
    pub link: Option<String>,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channel_item::Entity")]
    ChannelItem,
    #[sea_orm(has_many = "super::listening_state::Entity")]
    ListeningState,
}

impl Related<super::channel_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChannelItem.def()
    }
}

impl Related<super::listening_state::Entity> for Entity {
    fn to() -> RelationDef {
        super::channel_item::Relation::ListeningState.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::channel_item::Relation::Channel.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
