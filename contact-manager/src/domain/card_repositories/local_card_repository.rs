use anyhow::Result;

use crate::domain::{Card, CardRepository};

pub struct LocalCardRepository;

impl CardRepository for LocalCardRepository {
    fn create(_card: Card) -> Result<()> {
        todo!();
    }

    fn read(_id: String) -> Result<Card> {
        todo!()
    }

    fn read_all() -> Result<Vec<Card>> {
        todo!()
    }

    fn update(_card: Card) -> Result<()> {
        todo!()
    }

    fn delete(_id: String) -> Result<()> {
        todo!()
    }
}
