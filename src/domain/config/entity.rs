use anyhow::{anyhow, Context, Error, Result};
use log::{debug, trace};
use serde::Deserialize;
use shellexpand;
use std::{collections::HashMap, convert::TryFrom, env, fs, path::PathBuf, thread};
use toml;

use crate::output::utils::run_cmd;

const DEFAULT_PAGE_SIZE: usize = 10;

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Account {
    // TODO: rename with `from`
    pub name: Option<String>,
    pub downloads_dir: Option<PathBuf>,
    pub signature_delimiter: Option<String>,
    pub signature: Option<String>,
    pub default_page_size: Option<usize>,
    pub watch_cmds: Option<Vec<String>>,
    pub default: Option<bool>,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_starttls: Option<bool>,
    pub imap_insecure: Option<bool>,
    pub imap_login: String,
    pub imap_passwd_cmd: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_starttls: Option<bool>,
    pub smtp_insecure: Option<bool>,
    pub smtp_login: String,
    pub smtp_passwd_cmd: String,
}

pub type AccountsMap = HashMap<String, Account>;

/// Represents the whole config file.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    // TODO: rename with `from`
    pub name: String,
    pub downloads_dir: Option<PathBuf>,
    pub notify_cmd: Option<String>,
    /// Option to override the default signature delimiter "`--\n `".
    pub signature_delimiter: Option<String>,
    pub signature: Option<String>,
    pub default_page_size: Option<usize>,
    pub watch_cmds: Option<Vec<String>>,
    #[serde(flatten)]
    pub accounts: HashMap<String, Account>,
}

impl Config {
    fn path_from_xdg() -> Result<PathBuf> {
        let path = env::var("XDG_CONFIG_HOME").context("cannot find `XDG_CONFIG_HOME` env var")?;
        let mut path = PathBuf::from(path);
        path.push("himalaya");
        path.push("config.toml");

        Ok(path)
    }

    fn path_from_xdg_alt() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let mut path: PathBuf = env::var(home_var)
            .context(format!("cannot find `{}` env var", home_var))?
            .into();
        path.push(".config");
        path.push("himalaya");
        path.push("config.toml");

        Ok(path)
    }

    fn path_from_home() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let mut path: PathBuf = env::var(home_var)
            .context(format!("cannot find `{}` env var", home_var))?
            .into();
        path.push(".himalayarc");

        Ok(path)
    }

    pub fn path() -> Result<PathBuf> {
        let path = Self::path_from_xdg()
            .or_else(|_| Self::path_from_xdg_alt())
            .or_else(|_| Self::path_from_home())
            .context("cannot find config path")?;

        Ok(path)
    }

    /// Returns the account by the given name.
    /// If `name` is `None`, then the default account is returned.
    pub fn find_account_by_name(&self, name: Option<&str>) -> Result<&Account> {
        match name {
            Some("") | None => self
                .accounts
                .iter()
                .find(|(_, account)| account.default.unwrap_or(false))
                .map(|(_, account)| account)
                .ok_or_else(|| anyhow!("cannot find default account")),
            Some(name) => self
                .accounts
                .get(name)
                .ok_or_else(|| anyhow!(format!("cannot find account `{}`", name))),
        }
    }

    /// Returns the path to the given filename in the download directory.
    /// You can imagine this as:
    /// ```skip
    /// Account-specifique-download-dir-path + Attachment-Filename
    /// ```
    pub fn downloads_filepath(&self, account: &Account, filename: &str) -> PathBuf {
        account
            .downloads_dir
            .as_ref()
            .and_then(|dir| dir.to_str())
            .and_then(|dir| shellexpand::full(dir).ok())
            .map(|dir| PathBuf::from(dir.to_string()))
            .unwrap_or(
                self.downloads_dir
                    .as_ref()
                    .and_then(|dir| dir.to_str())
                    .and_then(|dir| shellexpand::full(dir).ok())
                    .map(|dir| PathBuf::from(dir.to_string()))
                    .unwrap_or(env::temp_dir()),
            )
            .join(filename)
    }

    /// This is a little helper-function like which uses the the name and email
    /// of the account to create a valid address for the header of the headers
    /// of a msg.
    ///
    /// # Hint
    /// If the name includes some special characters like a whitespace, comma or semicolon, then
    /// the name will be automatically wrapped between two `"`.
    ///
    /// # Exapmle
    /// ```
    /// use himalaya::config::model::{Account, Config};
    ///
    /// fn main() {
    ///     let config = Config::default();
    ///
    ///     let normal_account = Account::new(Some("Acc1"), "acc1@mail.com");
    ///     // notice the semicolon in the name!
    ///     let special_account = Account::new(Some("TL;DR"), "acc2@mail.com");
    ///
    ///     // -- Expeced outputs --
    ///     let expected_normal = Account {
    ///         name: Some("Acc1".to_string()),
    ///         email: "acc1@mail.com".to_string(),
    ///         .. Account::default()
    ///     };
    ///
    ///     let expected_special = Account {
    ///         name: Some("\"TL;DR\"".to_string()),
    ///         email: "acc2@mail.com".to_string(),
    ///         .. Account::default()
    ///     };
    ///
    ///     assert_eq!(config.address(&normal_account), "Acc1 <acc1@mail.com>");
    ///     assert_eq!(config.address(&special_account), "\"TL;DR\" <acc2@mail.com>");
    /// }
    /// ```
    pub fn address(&self, account: &Account) -> String {
        let name = account.name.as_ref().unwrap_or(&self.name);
        let has_special_chars = "()<>[]:;@.,".contains(|special_char| name.contains(special_char));

        if name.is_empty() {
            format!("{}", account.email)
        } else if has_special_chars {
            // so the name has special characters => Wrap it with '"'
            format!("\"{}\" <{}>", name, account.email)
        } else {
            format!("{} <{}>", name, account.email)
        }
    }

    pub fn run_notify_cmd<S: AsRef<str>>(&self, subject: S, sender: S) -> Result<()> {
        let subject = subject.as_ref();
        let sender = sender.as_ref();

        let default_cmd = format!(r#"notify-send "📫 {}" "{}""#, sender, subject);
        let cmd = self
            .notify_cmd
            .as_ref()
            .map(|cmd| format!(r#"{} {:?} {:?}"#, cmd, subject, sender))
            .unwrap_or(default_cmd);

        run_cmd(&cmd).context("cannot run notify cmd")?;

        Ok(())
    }

    /// Returns the signature of the given acccount in combination witht the sigantion delimiter.
    /// If the account doesn't have a signature, then the global signature is used.
    ///
    /// # Example
    /// ```
    /// use himalaya::config::model::{Config, Account};
    ///
    /// fn main() {
    ///     let config = Config {
    ///         signature: Some("Global signature".to_string()),
    ///         .. Config::default()
    ///     };
    ///
    ///     // a config without a global signature
    ///     let config_no_global = Config::default();
    ///
    ///     let account1 = Account::new_with_signature(Some("Account Name"), "mail@address.com", Some("Cya"));
    ///     let account2 = Account::new(Some("Bruh"), "mail@address.com");
    ///
    ///     // Hint: Don't forget the default signature delimiter: '\n-- \n'
    ///     assert_eq!(config.signature(&account1), Some("\n-- \nCya".to_string()));
    ///     assert_eq!(config.signature(&account2), Some("\n-- \nGlobal signature".to_string()));
    ///     
    ///     assert_eq!(config_no_global.signature(&account2), None);
    /// }
    /// ```
    pub fn signature(&self, account: &Account) -> Option<String> {
        let default_sig_delim = String::from("-- \n");
        let sig_delim = account
            .signature_delimiter
            .as_ref()
            .or_else(|| self.signature_delimiter.as_ref())
            .unwrap_or(&default_sig_delim);
        let sig = account
            .signature
            .as_ref()
            .or_else(|| self.signature.as_ref());
        sig.and_then(|sig| shellexpand::full(sig).ok())
            .map(|sig| sig.to_string())
            .and_then(|sig| fs::read_to_string(sig).ok())
            .or_else(|| sig.map(|sig| sig.to_owned()))
            .map(|sig| format!("\n{}{}", sig_delim, sig))
    }

    pub fn default_page_size(&self, account: &Account) -> usize {
        account
            .default_page_size
            .as_ref()
            .or_else(|| self.default_page_size.as_ref())
            .or(Some(&DEFAULT_PAGE_SIZE))
            .unwrap()
            .to_owned()
    }

    pub fn exec_watch_cmds(&self, account: &Account) -> Result<()> {
        let cmds = account
            .watch_cmds
            .as_ref()
            .or_else(|| self.watch_cmds.as_ref())
            .map(|cmds| cmds.to_owned())
            .unwrap_or_default();

        thread::spawn(move || {
            debug!("batch execution of {} cmd(s)", cmds.len());
            cmds.iter().for_each(|cmd| {
                debug!("running command {:?}…", cmd);
                let res = run_cmd(cmd);
                debug!("{:?}", res);
            })
        });

        Ok(())
    }
}

impl TryFrom<Option<&str>> for Config {
    type Error = Error;

    fn try_from(path: Option<&str>) -> Result<Self, Self::Error> {
        debug!("init config from `{:?}`", path);
        let path = path.map(|s| s.into()).unwrap_or(Config::path()?);
        let content = fs::read_to_string(path).context("cannot read config file")?;
        let config = toml::from_str(&content).context("cannot parse config file")?;
        trace!("{:#?}", config);
        Ok(config)
    }
}

// FIXME: tests
// #[cfg(test)]
// mod tests {
//     use crate::domain::{account::entity::Account, config::entity::Config};

//     // a quick way to get a config instance for testing
//     fn get_config() -> Config {
//         Config {
//             name: String::from("Config Name"),
//             ..Config::default()
//         }
//     }

//     #[test]
//     fn test_find_account_by_name() {
//         let mut config = get_config();

//         let account1 = Account::new(None, "one@mail.com");
//         let account2 = Account::new(Some("Two"), "two@mail.com");

//         // add some accounts
//         config.accounts.insert("One".to_string(), account1.clone());
//         config.accounts.insert("Two".to_string(), account2.clone());

//         let ret1 = config.find_account_by_name(Some("One")).unwrap();
//         let ret2 = config.find_account_by_name(Some("Two")).unwrap();

//         assert_eq!(*ret1, account1);
//         assert_eq!(*ret2, account2);
//     }

//     #[test]
//     fn test_address() {
//         let config = get_config();

//         let account1 = Account::new(None, "one@mail.com");
//         let account2 = Account::new(Some("Two"), "two@mail.com");
//         let account3 = Account::new(Some("TL;DR"), "three@mail.com");
//         let account4 = Account::new(Some("TL,DR"), "lol@mail.com");
//         let account5 = Account::new(Some("TL:DR"), "rofl@mail.com");
//         let account6 = Account::new(Some("TL.DR"), "rust@mail.com");

//         assert_eq!(&config.address(&account1), "Config Name <one@mail.com>");
//         assert_eq!(&config.address(&account2), "Two <two@mail.com>");
//         assert_eq!(&config.address(&account3), "\"TL;DR\" <three@mail.com>");
//         assert_eq!(&config.address(&account4), "\"TL,DR\" <lol@mail.com>");
//         assert_eq!(&config.address(&account5), "\"TL:DR\" <rofl@mail.com>");
//         assert_eq!(&config.address(&account6), "\"TL.DR\" <rust@mail.com>");
//     }
// }
