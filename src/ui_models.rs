#[derive(Debug, Clone, PartialEq)]
pub struct ChannelItem {
    pub ordering: i32,
    pub channel_id: i32,
    pub title: Option<String>,
    pub link: Option<String>,
    pub source: Option<String>,
    pub enclosure: String,
    pub description: Option<String>,
    pub guid: Option<String>,
    pub pub_date: Option<String>,
    pub listening_state: Option<ListeningState>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListeningState {
    pub time: f64,
    pub finished: bool,

}
