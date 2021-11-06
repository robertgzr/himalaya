use anyhow::Result;
use chrono::Utc;
use reqwest::blocking::Client;

use everest::domain::{card_repositories::RemoteCardRepository, Card, CardRepository};

#[test]
fn test_remote_card_repository() -> Result<()> {
    let host = "http://localhost:5232";
    let client = Client::new();
    let repository = RemoteCardRepository::new(host, &client)?;

    let id = "8f16d8b5-7e3a-6cd9-fa49-fc6cea65db2a";
    let card = Card {
        id: id.to_string(),
        date: Utc::now(),
        raw: [
            "BEGIN:VCARD",
            "VERSION:3.0",
            &format!("UID:{}", id),
            "EMAIL:test@mail.com",
            "FN:Test",
            "N:Nom;Prenom;;;",
            "ORG:Test",
            "TEL;TYPE=pref:06 06 06 06 06",
            "END:VCARD",
            "",
        ]
        .join("\r\n"),
    };

    repository.create(&card)?;
    let expected_card = repository.read(id)?;
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.raw, card.raw);

    repository.delete(&card.id)?;
    let res = repository.read(id);
    assert_eq!(
        res.unwrap_err().to_string(),
        format!(r#"cannot get card "{}""#, id)
    );

    Ok(())
}
