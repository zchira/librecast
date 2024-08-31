// use crate::entity::channel::Entity as ChannelEntity;
// use crate::entity::channel::Model as ChannelModel;
use crate::entity;
use crate::podcasts_model::PodcastsModel;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr};
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

                let items = channel.items().iter().map(|i| {
                    entity::channel_item::ActiveModel {
                        id: ActiveValue::NotSet,
                        channel_id: ActiveValue::set(channel_id),
                        title: ActiveValue::set(i.title().map(|t| t.to_string())),
                        // link: ActiveValue::set(i.link().map(|l| l.to_string())),
                        link: ActiveValue::set(Some(podcast_url.to_string())), // atom:link
                        source: ActiveValue::set(i.source().map(|s| s.url.to_string())),
                        enclosure: ActiveValue::set(i.enclosure().map(|e| e.url.to_string())),
                        description: ActiveValue::set(i.description().map(|d| d.to_string())),
                        guid: ActiveValue::set(i.guid().map(|g| g.value.clone())),
                        pub_date: ActiveValue::set(i.pub_date().map(|d| d.to_string())),
                    }
                });

                let _ = entity::channel_item::Entity::delete_many().filter(entity::channel_item::Column::ChannelId.eq(channel_id)).exec(&db).await;

                let _ = entity::channel_item::Entity::insert_many(items).exec(&db).await;
                Ok(channel_id)
            },
            Err(e) => {
                // handle error opening channel
                println!("{}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Get all podcast items from channel with id `channel_id`
    pub async fn get_items_from_db(channel_id: i32, db: &DatabaseConnection) -> Result<Vec<rss::Item>, DbErr> {
        let items = entity::channel_item::Entity::find().filter(entity::channel_item::Column::ChannelId.eq(channel_id)).all(db).await?;
        let mut to_ret: Vec<rss::Item> = Default::default();

        items.iter().for_each(|i| {
            to_ret.push(i.into());
        });

        Ok(to_ret)
    }
}

impl From<&entity::channel_item::Model> for rss::Item {
    fn from(i: &entity::channel_item::Model) -> Self {
        let item = rss::Item {
            title: i.title.clone(),
            link: i.link.clone(),
            description: i.description.clone(),
            author: None,
            categories: Default::default(),
            comments: None,
            enclosure: i.enclosure.as_ref().map(|s| rss::Enclosure{
                url: s.to_string(),
                length: "".to_string(),
                mime_type: "".to_string(),
            }),
            guid: None,
            pub_date: i.pub_date.clone(),
            source: i.source.as_ref().map(|s| rss::Source { title: None, url: s.clone()}),
            content: None,
            extensions: Default::default(),
            itunes_ext: None,
            dublin_core_ext: None,
        };
        item
    }
}
