mod tg_helper;
mod reply;
mod db_helper;

use std::{fs, thread};

use futures::StreamExt;
use telegram_bot::*;
use mysql::Pool;
use crate::tg_helper::handle_replies;
use ts3_query::{QueryClient, MessageTarget};
use std::sync::mpsc;
use std::time::{Instant, Duration};
use std::sync::mpsc::{Sender, Receiver};
use std::convert::TryFrom;
use crate::db_helper::get_alias_from_telegram_id;

fn load_configuration(path: &str) -> (String, String, String, String, String, String, String, String, u16, String, String) {
    let raw_conf = fs::read_to_string(path).expect("Could not read configuration file!");
    let conf: serde_json::Value = serde_json::from_str(&raw_conf).expect("Configuration file is malformed");

    let bot_token = conf["bot_token"].as_str().expect("Configuration file does not contain bot_token!").to_string();
    let database_name = conf["database_name"].as_str().expect("Configuration file does not contain database_name!").to_string();
    let database_ip = conf["database_ip"].as_str().expect("Configuration file does not contain database_ip!").to_string();
    let database_user = conf["database_user"].as_str().expect("Configuration file does not contain database_user!").to_string();
    let database_password = conf["database_password"].as_str().expect("Configuration file does not contain database_password!").to_string();
    let tg_log_chatid = conf["tg_log_chatid"].as_str().expect("Configuration file does not contain tg_log_chatid!").to_string();

    let ts_ip = conf["ts_ip"].as_str().expect("Configuration file does not contain ts_ip!").to_string();
    let ts_query_port = conf["ts_query_port"].as_str().expect("Configuration file does not contain ts_query_port!").to_string();
    let ts_server_port = u16::try_from(conf["ts_server_port"].as_u64().expect("Configuration file does not contain ts_server_port!")).expect("ts_server_port is not a valid port number!");
    let ts_user = conf["ts_user"].as_str().expect("Configuration file does not contain ts_user!").to_string();
    let ts_password = conf["ts_password"].as_str().expect("Configuration file does not contain ts_password!").to_string();

    return (bot_token, database_name, database_ip, database_user, database_password, tg_log_chatid, ts_ip, ts_query_port, ts_server_port, ts_user, ts_password);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (bot_token, database_name, database_ip, database_user, database_password, tg_log_chatid, ts_ip, ts_query_port, ts_server_port, ts_user, ts_password) = load_configuration("configuration.json");
    let api = Api::new(bot_token);

    let db_pool = Pool::new(format!("mysql://{}:{}@{}/{}", database_user, database_password, database_ip, database_name)).expect("Database connection can't be established!");

    let (ts_sender, ts_receiver) : (Sender<(String, String)>, Receiver<(String, String)>) = mpsc::channel();

    let _ = thread::spawn(move || {
        let mut client = QueryClient::new(format!("{}:{}", ts_ip, ts_query_port)).unwrap();
        client.login(ts_user, ts_password).unwrap();
        client.select_server_by_port(ts_server_port).unwrap();

        let mut last_msg = Instant::now();
        loop {
            let recv = ts_receiver.try_recv();
            if recv.is_err() {
                if last_msg.elapsed().as_secs() > 240 {     // We will be disconnected from the server if we don't do anything for 5 minutes
                    let _ = client.ping();                  // So we ping it roughly every 4 minutes
                    last_msg = Instant::now();
                    continue;
                }
                thread::sleep(Duration::from_millis(100));
            } else {
                let recv = recv.unwrap();
                let _ = client.rename(recv.0);
                let _ = client.send_message(MessageTarget::Server, recv.1);
            }
        }
    });

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("{} {} in {}: {}", message.clone().from.first_name, message.clone().from.last_name.unwrap_or("".to_string()), message.clone().chat.id(), data);
                if message.chat.id().to_string().eq(&tg_log_chatid) {
                    let mut users_name =
                        get_alias_from_telegram_id(
                            db_pool.get_conn().unwrap(),
                            &message.clone().from.id.to_string(),
                            Some(
                                (
                                    message.clone().from.first_name,
                                    message.clone().from.last_name.unwrap_or("".to_string()),
                                    message.clone().from.username.unwrap_or("".to_string())
                                )
                            )
                        );
                    users_name = if users_name.eq(&"".to_string()) { format!("{} {}", message.clone().from.first_name, message.clone().from.last_name.unwrap_or("".to_string())) } else { users_name};
                    let _ = ts_sender.send((users_name, data.to_string()));
                    let _ = api.send(message.delete()).await;
                }
            }

            handle_replies(&api, db_pool.get_conn().unwrap(), &message).await;
        }
    }
    Ok(())
}