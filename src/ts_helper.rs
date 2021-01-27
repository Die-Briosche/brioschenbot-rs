use std::time::{Instant, Duration};
use std::thread;
use ts3_query::{MessageTarget, QueryClient};
use std::sync::mpsc::{Receiver, Sender};
use crate::db_helper::get_tsname_from_userid;
use mysql::PooledConn;
use std::borrow::BorrowMut;

pub enum TSCommand {
    ServerMessageSend(String, String),
    ServerPokeAll(String, String),
    ServerUsersOnline(Sender<String>),
    UserKick(String, String, String),
    UserChannelKick(String, String, String),
    UserPoke(String, String, String),
    Surprise(Sender<String>),
}

pub fn start_ts_handler(mut client: QueryClient, mut db_conn: PooledConn, ts_receiver: Receiver<TSCommand>, surprise_target: String) {
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
            //let msg = recv.unwrap();
            match recv.unwrap() {
                TSCommand::ServerMessageSend(username, message) => {
                    let _ = client.rename(&username);
                    let _ = client.send_message(MessageTarget::Server, message);
                    let _ = client.rename("BrioschenBot");
                },
                TSCommand::ServerPokeAll(username, message) => {
                    let _ = client.rename(username);

                    let online_clients = client.online_clients().unwrap();
                    for ts_user in online_clients {
                        let _ = client.poke_client(ts_user.clid, message.clone());
                    }

                    let _ = client.rename("BrioschenBot");
                },
                TSCommand::ServerUsersOnline(answer_sender) => {
                    let mut users_string = String::from("Currently online:\n\n");

                    let online_clients = client.online_clients().unwrap();
                    for ts_user in online_clients {
                        if ts_user.client_type != 1 {
                            users_string += &*format!("{}\n", ts_user.client_nickname).to_string();
                        }
                    }
                    let _ = answer_sender.send(users_string);
                },
                TSCommand::UserPoke(target_username, username, message) => {
                    let _ = client.rename(username);

                    let online_clients = client.online_clients().unwrap();
                    for ts_user in online_clients {
                        if ts_user.client_nickname.eq(&target_username.clone()) {
                            let _ = client.poke_client(ts_user.clid, message.clone());
                        }
                    }

                    let _ = client.rename("BrioschenBot");
                },
                TSCommand::UserKick(target_username, username, message) => {
                    let _ = client.rename(username);

                    let online_clients = client.online_clients().unwrap();
                    for ts_user in online_clients {
                        println!("{} - {}", target_username, ts_user.client_nickname);
                        if target_username.eq(&ts_user.client_nickname) {
                            let _ = client.kick_client(ts_user.clid, true, Some(&message.clone()));
                        }
                    }

                    let _ = client.rename("BrioschenBot");
                },
                TSCommand::UserChannelKick(target_username, username, message) => {
                    let _ = client.rename(username);

                    let online_clients = client.online_clients().unwrap();
                    for ts_user in online_clients {
                        if ts_user.client_nickname.eq(&target_username.clone()) {
                            let _ = client.kick_client(ts_user.clid, false, Some(&message.clone()));
                        }
                    }

                    let _ = client.rename("BrioschenBot");
                },
                TSCommand::Surprise(answer_sender) => {
                    let online_clients = client.online_clients();   // TODO use online_clients_full() once it doesn't regularly fail anymore
                    let tsname = get_tsname_from_userid(db_conn.borrow_mut(), &surprise_target.clone());

                    if online_clients.is_ok() {
                        for ts_user in online_clients.unwrap() {

                            if ts_user.client_nickname.eq(&tsname) {
                                let _ = client.kick_client(ts_user.clid, true, Some("Surprise!"));
                                let _ = answer_sender.send("Surprisingly silent!".to_string());
                            }
                        }
                    }
                }
            }


        }
    }
}