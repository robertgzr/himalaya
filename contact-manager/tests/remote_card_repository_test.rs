use anyhow::Result;
use clap;
use env_logger;
use reqwest::Client;
use std::env;
use tokio;

use everest::domain::card_repositories::remote_card_repository::*;

#[tokio::test]
async fn main() -> Result<()> {
    let host = "http://localhost:5232";
    let client = Client::new();

    let path = String::from("/");
    let path = fetch_current_user_principal_url(host, path, &client).await?;
    let path = fetch_addressbook_home_set_url(host, path, &client).await?;
    let path = fetch_addressbook_url(host, path, &client).await?;

    assert_eq!("/", path);
    Ok(())
}
