use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub date: DateTime<Utc>,
}
