use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub date: DateTime<Utc>,
    pub raw: String,
}
