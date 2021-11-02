use anyhow::Result;

use crate::domain::Card;

pub trait CardRepository {
    fn create(card: Card) -> Result<()>;
    fn read(id: String) -> Result<Card>;
    fn read_all() -> Result<Vec<Card>>;
    fn update(card: Card) -> Result<()>;
    fn delete(id: String) -> Result<()>;
}
