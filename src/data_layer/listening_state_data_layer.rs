use sea_orm::ActiveValue;
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::entity::listening_state;
use crate::entity::listening_state::Entity as ListeningStateEntity;
use crate::entity::listening_state::ActiveModel as ListeningStateModel;


pub struct ListeningStateDataLayer {}

impl ListeningStateDataLayer {
    async fn create_listenitg_state_for_item(db: DatabaseConnection, enclosure_url: String, channel_id: i32, time: f32) {
        let model = ListeningStateModel {
            id: ActiveValue::NotSet,
            channel_id: ActiveValue::set(channel_id),
            channel_item_enclosure: ActiveValue::set(enclosure_url),
            time: ActiveValue::set(time),
            finished: ActiveValue::set(false),
        };

        let _ = ListeningStateEntity::insert(model).exec(&db).await;


    }

    pub async fn mark_item_as_finished(db: DatabaseConnection, enclosure_url: String, channel_id: i32) -> Result<(), sea_orm::DbErr> {
        let res = ListeningStateEntity::find()
            .filter(listening_state::Column::ChannelId.eq(channel_id))
            .filter(listening_state::Column::ChannelItemEnclosure.eq(enclosure_url))
            .one(&db).await?;

        match res {
            Some(i) => {
                let mut m: ListeningStateModel = i.into();
                m.finished = ActiveValue::set(true);
                m.time = ActiveValue::set(0.0);
                let res2 = ListeningStateEntity::update(m).exec(&db).await?;
                Ok(())
            },
            None => {
                Ok(())
            },
        }
    }

    pub async fn update_current_time_for_item(db: DatabaseConnection, enclosure_url: String, channel_id: i32, time: f32) -> Result<(), sea_orm::DbErr> {
        let res = ListeningStateEntity::find()
            .filter(listening_state::Column::ChannelId.eq(channel_id))
            .filter(listening_state::Column::ChannelItemEnclosure.eq(&enclosure_url))
            .one(&db).await?;

        match res {
            Some(i) => {
                let mut m: ListeningStateModel = i.into();
                m.finished = ActiveValue::set(false);
                m.time = ActiveValue::set(time);
                let _ = ListeningStateEntity::update(m).exec(&db).await?;
                Ok(())
            },
            None => {
                ListeningStateDataLayer::create_listenitg_state_for_item(db, enclosure_url, channel_id, time).await;
                Ok(())
            },
        }
    }

}
