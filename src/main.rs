mod tg_helper;
mod reply;

use std::fs;

use futures::StreamExt;
use telegram_bot::*;
use mysql::Pool;
use crate::tg_helper::handle_replies;

fn load_configuration(path: &str) -> (String, String, String, String, String) {
    let raw_conf = fs::read_to_string(path).expect("Could not read configuration file!");
    let conf: serde_json::Value = serde_json::from_str(&raw_conf).expect("Configuration file is malformed");

    let bot_token = conf["bot_token"].as_str().expect("Configuration file does not contain an bot_token!").to_string();
    let database_name = conf["database_name"].as_str().expect("Configuration file does not contain database_name!").to_string();
    let database_ip = conf["database_ip"].as_str().expect("Configuration file does not contain database_ip!").to_string();
    let database_user = conf["database_user"].as_str().expect("Configuration file does not contain database_user!").to_string();
    let database_password = conf["database_password"].as_str().expect("Configuration file does not contain database_password!").to_string();

    return (bot_token, database_name, database_ip, database_user, database_password);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (bot_token, database_name, database_ip, database_user, database_password) = load_configuration("configuration.json");
    let api = Api::new(bot_token);

    let db_pool = Pool::new(format!("mysql://{}:{}@{}/{}", database_user, database_password, database_ip, database_name)).expect("Database connection can't be established!");


    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            handle_replies(&api, db_pool.get_conn().unwrap(), &message).await;
        }
    }
    Ok(())
}