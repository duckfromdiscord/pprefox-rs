pub mod http;
pub mod json;

use directories::ProjectDirs;
use registry::{Hive, RegKey, Security};
use std::path::PathBuf;
use utfx::U16CString;

#[derive(Debug)]
pub enum Error {
    ErrorCreatingProjectKey,
    ErrorWritingProjectKey,
    FirefoxNotFound,
    InvalidJsonPath,
    ErrorWritingConfigData,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => {
                write!(f, "pprefox-rs error")
            }
        }
    }
}

impl std::error::Error for Error {}

fn create_subkey_if_not_exist(rk: RegKey, name: &str) -> Result<RegKey, registry::key::Error> {
    match rk.open(name, Security::Read | Security::Write) {
        Ok(rk) => Ok(rk),
        Err(err) => match err {
            registry::key::Error::NotFound(..) => rk.create(name, Security::Read | Security::Write),
            _ => Err(err),
        },
    }
}

fn firefox_root() -> Result<RegKey, registry::key::Error> {
    Hive::CurrentUser.open(r"Software\\Mozilla\\", Security::Read | Security::Write)
}

fn create_nmh(rk: RegKey) -> Result<RegKey, Error> {
    match create_subkey_if_not_exist(rk, "NativeMessagingHosts") {
        Ok(nmh) => create_subkey_if_not_exist(nmh, "pprefox_rs")
            .map_err(|_| Error::ErrorCreatingProjectKey),
        Err(_) => Err(Error::ErrorCreatingProjectKey),
    }
}

/// Creates the registry keys for Firefox to detect the program as a NativeMessage host
pub fn firefox_write(json_location: &str) -> Result<(), Error> {
    match firefox_root() {
        Ok(rk) => {
            let nmh = create_nmh(rk);
            match nmh {
                Ok(nmh) => match U16CString::from_str(json_location) {
                    Err(_) => Err(Error::InvalidJsonPath),
                    Ok(ws) => nmh
                        .set_value("", &registry::Data::String(ws))
                        .map_err(|_| Error::ErrorWritingProjectKey),
                },
                Err(_) => Err(Error::ErrorCreatingProjectKey),
            }
        }
        Err(_) => Err(Error::FirefoxNotFound),
    }
}

use serde_derive::*;
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Host {
    pub name: String,
    pub description: String,
    pub path: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub allowed_extensions: Vec<String>,
}

/// Creates the host JSON and the batch file it points to. Returns the path to the JSON.
pub fn nmh_files_setup(batch_contents: &str) -> Result<PathBuf, Error> {
    // Directories to store the config and serve script
    let proj_dirs = ProjectDirs::from("", "duck", "pprefox_rs")
        .expect("Could not initialize project directories for pprefox_rs");
    match std::fs::create_dir_all(proj_dirs.data_dir()) {
        Err(_) => Err(Error::ErrorWritingConfigData),
        Ok(_) => {
            let host_path = proj_dirs.data_dir().join("host.json");
            let script_path = proj_dirs.data_dir().join("nmhhost.bat");
            let script_result = std::fs::write(script_path.clone(), batch_contents)
                .map_err(|_| Error::ErrorWritingConfigData);
            let script_path = script_path.to_str().unwrap().to_string();
            if script_result.is_err() {
                return Err(Error::ErrorWritingConfigData);
            }
            let host = Host {
                name: "pprefox_rs".to_string(),
                description: "pprefox_rs".to_string(),
                path: script_path.clone(),
                _type: "stdio".to_string(),
                allowed_extensions: vec!["pprefox@duckfromdiscord.github.io".to_string()],
            };
            let host_file = serde_json::ser::to_string_pretty(&host).unwrap();
            let host_result = std::fs::write(host_path.clone(), host_file)
                .map_err(|_| Error::ErrorWritingConfigData);
            host_result.and(script_result).map(|_| host_path)
        }
    }
}
