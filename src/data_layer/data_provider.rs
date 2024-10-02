// use crate::entity::channel::Entity as ChannelEntity;
// use crate::entity::channel::Model as ChannelModel;
use crate::entity::{self, channel_item, listening_state};
use crate::podcasts_model::PodcastsModel;
use crate::ui_models;
use chrono::FixedOffset;
use sea_orm::{ActiveValue, Database, DatabaseConnection, DbErr, QueryOrder, QuerySelect};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::error::Error;
use std::io::ErrorKind;

pub struct DataProvider {}

impl DataProvider {
    /// Fetch data from provided url,
    /// and write data in db
    pub async fn fetch_data(podcast_url: String, selected_channel_id: i32, db: DatabaseConnection) -> Result<i32, Box<dyn Error>> {
        match PodcastsModel::get_channel_from_url(&podcast_url).await.map_err(|_| std::io::Error::new(ErrorKind::Other, "")) {
            Ok(channel) => {
                use entity::channel::{ Entity, ActiveModel };
                let am: ActiveModel = ActiveModel {
                    title: ActiveValue::set(Some(channel.title().to_string())),
                    link: ActiveValue::set(Some(podcast_url.to_string())),
                    description: ActiveValue::set(Some(channel.description().to_string())),
                    id: ActiveValue::set(selected_channel_id) // ActiveValue::NotSet
                };

                let exist = Entity::find().filter(entity::channel::Column::Link.eq(channel.link())).one(&db).await.unwrap();

                let channel_id = if let Some(exist) = exist {
                    exist.id
                } else {
                    let res = Entity::update(am).exec(&db).await.unwrap();
                    res.id
                };

                let mut order = 0;
                let items: Vec<_> = channel.items().iter().map(|i| {
                    order = order + 1;
                    let d: Option<chrono::DateTime<FixedOffset>> = match chrono::DateTime::parse_from_rfc2822(i.pub_date().unwrap_or(Default::default())) {
                        Ok(s) => Some(s),
                        Err(_) => None,
                    };
                    entity::channel_item::ActiveModel {
                        ordering: ActiveValue::set(order),
                        channel_id: ActiveValue::set(channel_id),
                        title: ActiveValue::set(i.title().map(|t| t.to_string())),
                        // link: ActiveValue::set(i.link().map(|l| l.to_string())),
                        link: ActiveValue::set(Some(podcast_url.to_string())), // atom:link
                        source: ActiveValue::set(i.source().map(|s| s.url.to_string())),
                        enclosure: ActiveValue::set(i.enclosure().map(|e| e.url.to_string()).unwrap()),
                        description: ActiveValue::set(i.description().map(|d| d.to_string())),
                        guid: ActiveValue::set(i.guid().map(|g| g.value.clone())),
                        pub_date: ActiveValue::set(d)
                    }
                }).collect();


                let r = entity::channel_item::Entity::delete_many().filter(entity::channel_item::Column::ChannelId.eq(channel_id)).exec(&db).await;

                for c in items.chunks(500) {
                    let r = entity::channel_item::Entity::insert_many(c.to_vec()).exec(&db).await;
                    // handle error opening channel
                }

                Ok(channel_id)
            },
            Err(e) => {
                // handle error opening channel
                Err(Box::new(e))
            }
        }
    }

    /// Get all podcast items from channel with id `channel_id`
    pub async fn get_items_from_db(channel_id: i32, db: &DatabaseConnection) -> Result<Vec<ui_models::ChannelItem>, DbErr> {
        let items = entity::channel_item::Entity::find()
            .filter(entity::channel_item::Column::ChannelId.eq(channel_id))
            .order_by_desc(channel_item::Column::PubDate)
            .order_by_asc(channel_item::Column::Ordering)
            .find_with_related(listening_state::Entity)
            .all(db).await?;

        let mut to_ret: Vec<ui_models::ChannelItem> = Default::default();

        items.iter().for_each(|i| {
            to_ret.push(i.into());
        });

        Ok(to_ret)
    }
}

impl From<&(entity::channel_item::Model, Vec<listening_state::Model>)> for ui_models::ChannelItem {
    fn from(entry: &(entity::channel_item::Model, Vec<listening_state::Model>)) -> Self {
        let i = entry.0.clone();
        let listening_state = if entry.1.len() > 0 {
            let ls = entry.1.get(0);
            match ls {
                Some(e) => Some(ui_models::ListeningState {
                    time: e.time,
                    finished: e.finished,
                }),
                None => None,
            }
        } else {
            None
        };

        let item = ui_models::ChannelItem {
            title: i.title.clone(),
            link: i.link.clone(),
            description: i.description.clone(),
            enclosure: i.enclosure.to_string(),
            guid: None,
            pub_date: i.pub_date.clone(),
            source: i.source,
            ordering: i.ordering,
            channel_id: i.channel_id,
            listening_state,
        };
        item
    }
}

// #[tokio::test]
async fn test_get_items_with_state() -> Result<(), DbErr> {

    let home = std::env::var("HOME").unwrap();
    let connection = std::env::var("DATABASE_URL").unwrap_or(format!("sqlite://{}/.librecast.db?mode=rwc", home));
    let db: DatabaseConnection = Database::connect(connection).await?;

    let enclosure_url = "https://media.transistor.fm/dd1e8e10/db7f2e6d.mp3".to_string();
    crate::data_layer::listening_state_data_layer::ListeningStateDataLayer::create_listenitg_state_for_item(db.clone(), enclosure_url, 2, 5.0).await;


    let enclosure_url = "https://media.transistor.fm/59bdd3e8/20064bdc.mp3".to_string();
    crate::data_layer::listening_state_data_layer::ListeningStateDataLayer::create_listenitg_state_for_item(db.clone(), enclosure_url.clone(), 2, 5.0).await;
    let _ = crate::data_layer::listening_state_data_layer::ListeningStateDataLayer::mark_item_as_finished(db.clone(), enclosure_url, 2).await;


    // println!("channel: {:#?}", channel);
    // println!("-----------");
    // println!("{:#?}", ext);
    Ok(())
}

#[tokio::test]
async fn test_join() -> Result<(), DbErr> {

    let home = std::env::var("HOME").unwrap();
    let connection = std::env::var("DATABASE_URL").unwrap_or(format!("sqlite://{}/.librecast.db?mode=rwc", home));
    let db: DatabaseConnection = Database::connect(connection).await?;

    let res = DataProvider::get_items_from_db(2, &db).await?;
    println!("RES!: {:#?}", res);


    Ok(())
}
