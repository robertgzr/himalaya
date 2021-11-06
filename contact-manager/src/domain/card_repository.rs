use anyhow::Result;

use crate::domain::Card;

pub trait CardRepository {
    fn create(&self, card: &Card) -> Result<()>;
    fn read(&self, id: &str) -> Result<Card>;
    fn read_all(&self) -> Result<Vec<Card>>;
    fn update(&self, card: &Card) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
}
