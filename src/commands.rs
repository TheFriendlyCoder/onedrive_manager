//! Entrypoint functions for all of our CLI commands
use crate::auth::{
    get_auth_data, get_auth_url, get_oauth_token_from_browser, parse_token, refresh_auth_data,
    REDIRECT_URI,
};
use crate::configfile::Configuration;
use futures::executor;
use onedrive_api::{DriveLocation, ItemLocation, OneDrive};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

type MyResult<T> = Result<T, Box<dyn Error>>;

/// Path to folder containing configuration data for the app
fn config_folder() -> PathBuf {
    // To Keep things simple we just 'expect' the users home folder
    // to exist on every platform supported by the tool, so we just
    // panic if we run into some weird edge case here
    dirs::home_dir()
        .expect("Unable to resolve use home folder")
        .join(".onedrive_manager")
}

/// Path to the configuration file containing options that customize
/// the behavior of the application
fn config_file() -> PathBuf {
    config_folder().join("config.yml")
}

/// Entry point function for the "init" subcommand
///
/// The command prompts the user for authentication parameters to OneDrive
/// and then generates a configuration file in the curent folder named
/// config.yml containing the authentication tokens retrieved from the
/// OAuth provider
///
/// # Arguments
///
/// * `browser` - True if the user wants the browser to be automatically
///               launched by our app, and have the response from the
///               authentication request automatically intercepted
pub fn init_cmd(browser: bool) -> MyResult<()> {
    let response_url = match browser {
        true => {
            println!("Waiting for OneDrive authentication request in your browser...");
            println!("Reference URL: {}", get_auth_url());
            println!("Listening for response on: {}", REDIRECT_URI);

            get_oauth_token_from_browser()?
        }
        false => {
            println!("Open this URL in your browser: {}", get_auth_url());
            print!("Paste the response URL here: ");
            stdout().flush()?;
            let mut temp = String::new();
            stdin().read_line(&mut temp)?;
            temp
        }
    };

    let token = parse_token(&response_url)?;
    let auth = get_auth_data(&token)?;

    let config = Configuration {
        auth_token: auth.access_token,
        refresh_token: auth.refresh_token,
    };

    config.save(&config_file())?;

    Ok(())
}

/// Entrypoint method for the 'ls' subcommand
/// Shows a directory listing of the root OneDrive folder
pub fn ls_cmd() -> MyResult<()> {
    let mut config = Configuration::from_file(&config_file())?;

    let service = OneDrive::new(config.auth_token, DriveLocation::me());
    let src_item = ItemLocation::root();
    let result = executor::block_on(service.list_children(src_item));
    let children = match result {
        Ok(d) => d,
        Err(_) => {
            // If our first attempt to perform the operation fails, request a token
            // refresh from OneDrive and try again
            let temp = refresh_auth_data(&config.refresh_token)?;
            config.auth_token = temp.access_token;
            config.refresh_token = temp.refresh_token;
            config.save(&config_file())?;

            let drive = OneDrive::new(config.auth_token, DriveLocation::me());
            let item = ItemLocation::root();
            let a = drive.list_children(item);
            executor::block_on(a)?
        }
    };

    // Iterate through children and show their names to the user
    for i in children {
        println!("{}", i.name.unwrap());
    }
    Ok(())
}