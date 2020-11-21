use std::time::{Instant, Duration};
use std::thread;
use ts3_query::{MessageTarget, QueryClient};
use std::sync::mpsc::Receiver;

pub enum TSCommand {
    ServerMessageSend(String, String),
    ServerPokeAll(String, String),
    UserKick(String, String, String),
    UserChannelKick(String, String, String),
    UserPoke(String, String, String),

}

pub fn start_ts_handler(mut client: QueryClient, ts_receiver: Receiver<TSCommand>) {    // TODO Find a way to communicate back
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
            let msg = recv.unwrap();
            if let TSCommand::ServerMessageSend(username, message) = msg {
                let _ = client.rename(username);
                let _ = client.send_message(MessageTarget::Server, message);
                let _ = client.rename("BrioschenBot");
            } else if let TSCommand::ServerPokeAll(username, message) = msg {
                let _ = client.rename(username);

                let online_clients = client.online_clients().unwrap();
                for ts_user in online_clients {
                    let _ = client.poke_client(ts_user.clid, message.clone());
                }

                let _ = client.rename("BrioschenBot");
            } else if let TSCommand::UserPoke(target_username, username, message) = msg {
                let _ = client.rename(username);

                let online_clients = client.online_clients().unwrap();
                for ts_user in online_clients {
                    if ts_user.client_nickname.eq(&target_username.clone()) {
                        let _ = client.poke_client(ts_user.clid, message.clone());
                    }
                }

                let _ = client.rename("BrioschenBot");
            } else if let TSCommand::UserKick(target_username, username, message) = msg {
                let _ = client.rename(username);

                let online_clients = client.online_clients().unwrap();
                for ts_user in online_clients {
                    println!("{} - {}", target_username, ts_user.client_nickname);
                    if target_username.eq(&ts_user.client_nickname) {
                        let _ = client.kick_client(ts_user.clid, true, Some(&message.clone()));
                    }
                }

                let _ = client.rename("BrioschenBot");
            } else if let TSCommand::UserChannelKick(target_username, username, message) = msg {
                let _ = client.rename(username);

                let online_clients = client.online_clients().unwrap();
                for ts_user in online_clients {
                    if ts_user.client_nickname.eq(&target_username.clone()) {
                        let _ = client.kick_client(ts_user.clid, false, Some(&message.clone()));
                    }
                }

                let _ = client.rename("BrioschenBot");
            }


        }
    }
}