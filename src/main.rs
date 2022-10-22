mod tg_helper;
mod reply;
mod db_helper;
mod ts_helper;

use std::{env, thread};

use futures::StreamExt;
use telegram_bot::*;
use mysql::Pool;
use crate::tg_helper::{handle_replies, handle_commands, handle_tiktok};
use ts3_query::QueryClient;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use crate::db_helper::get_alias_from_telegram_id;
use crate::ts_helper::{start_ts_handler, TSCommand};

#[derive(Clone)]
pub struct Configuration {
    bot_token: String,
    database_name: String,
    database_ip: String,
    database_user: String,
    database_password: String,
    tg_log_chatid: String,
    ts_ip: String,
    ts_query_port: String,
    ts_server_port: u16,
    ts_user: String,
    ts_password: String,
    surprise_target: String,
    log_randomnum_exception: String
}

pub fn load_configuration() -> Configuration {
    Configuration {
        bot_token: env::var("BOT_TOKEN").expect("No BOT_TOKEN environment variable!"),
        database_name: env::var("DB_NAME").expect("No DB_NAME environment variable!"),
        database_ip: env::var("DB_IP").expect("No DB_IP environment variable!"),
        database_user: env::var("DB_USER").expect("No DB_USER environment variable!"),
        database_password: env::var("DB_PASSWORD").expect("No DB_PASSWORD environment variable!"),
        tg_log_chatid: env::var("TG_LOG_CHATID").expect("No TG_LOG_CHATID environment variable!"),

        ts_ip: env::var("TS_IP").expect("No TS_IP environment variable!"),
        ts_query_port: env::var("TS_QUERY_PORT").expect("No TS_QUERY_PORT environment variable!"),
        ts_server_port: env::var("TS_SERVER_PORT").expect("No TS_SERVER_PORT environment variable!").parse().expect("TS_SERVER_PORT is not a valid port number!"),
        ts_user: env::var("TS_USER").expect("No TS_USER environment variable!"),
        ts_password: env::var("TS_PASSWORD").expect("No TS_PASSWORD environment variable!"),

        surprise_target: env::var("SURPRISE_TARGET").expect("No SURPRISE_TARGET environment variable"),
        log_randomnum_exception: env::var("LOG_RANDNUM_EXCEPTION").expect("No LOG_RANDNUM_EXCEPTION environment variable"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let configuration = load_configuration();
    let api = Api::new(&configuration.bot_token);

    let db_pool = Pool::new(format!("mysql://{}:{}@{}/{}", configuration.database_user, configuration.database_password, configuration.database_ip, configuration.database_name)).expect("Database connection can't be established!");
    let ts_db_conn = db_pool.get_conn().unwrap();

    let (ts_sender, ts_receiver) : (Sender<TSCommand>, Receiver<TSCommand>) = mpsc::channel();

    let ts_conf = configuration.clone();
    let _ = thread::spawn(move || {
        let mut client = QueryClient::new(format!("{}:{}", ts_conf.ts_ip, ts_conf.ts_query_port)).unwrap();
        client.login(ts_conf.ts_user, ts_conf.ts_password).unwrap();
        client.select_server_by_port(ts_conf.ts_server_port).unwrap();

        start_ts_handler(client,  ts_db_conn, ts_receiver, ts_conf.surprise_target);
    });

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        if update.is_err() {
            println!("Update error'd out");
            continue;
        }
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("{} {} in {}: {}", message.clone().from.first_name, message.clone().from.last_name.unwrap_or("".to_string()), message.clone().chat.id(), data);
                db_helper::log_message(&mut db_pool.get_conn().unwrap(), message.clone());
                if message.chat.id().to_string().eq(&configuration.tg_log_chatid) {
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
                    if message.from.id.to_string().eq(&configuration.log_randomnum_exception) {
                        users_name += &rand::random::<u32>().to_string();
                    }
                    users_name += " (Log)";
                    let _ = ts_sender.send(TSCommand::ServerMessageSend(users_name, data.to_string()));
                    let _ = api.send(message.delete()).await;
                    continue;
                }
            }

            if handle_commands(&api, db_pool.get_conn().unwrap(), ts_sender.clone(), &message).await {
                continue;
            } else if handle_tiktok(&api, &message).await {
                continue;
            } else {
                handle_replies(&api, db_pool.get_conn().unwrap(), &message).await;
            }
        }
    }
    Ok(())
}