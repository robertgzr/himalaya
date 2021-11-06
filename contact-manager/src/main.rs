use anyhow::Result;
use clap;
use env_logger;
use reqwest::Client;
use std::env;

fn create_app<'a>() -> clap::App<'a, 'a> {
    clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .global_setting(clap::AppSettings::GlobalVersion)
}

fn main() -> Result<()> {
    // Init env logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "off"),
    );

    // let app = create_app();
    // let _m = app.get_matches();
    // Check completion command BEFORE entities and services initialization.
    // match compl_arg::matches(&m)? {
    //     Some(compl_arg::Command::Generate(shell)) => {
    //         return compl_handler::generate(create_app(), shell);
    //     }
    //     _ => (),
    // }

    Ok(())
}
