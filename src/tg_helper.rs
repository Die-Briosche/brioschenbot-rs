use telegram_bot::{Api, CanReplySendMessage, MessageKind, CanSendMessage, Message, InputFileUpload, CanSendPhoto, CanSendDocument, CanReplySendDocument, InputFileRef, CanDeleteMessage};
use mysql::PooledConn;
use mysql::prelude::Queryable;
use crate::reply::{Comparator, Reply, ReplyType};
use std::sync::mpsc::{Sender, Receiver};
use crate::ts_helper::TSCommand;
use regex::Regex;
use std::sync::mpsc;


pub async fn handle_replies(api: &Api, mut db_conn: PooledConn, message: &Message) -> bool {
    if let MessageKind::Text { ref data, .. } = message.kind {

        // The "ORDER BY rand()" is actually only needed for the random gifs. Should be replaced with a cleaner version that isn't as costly for big tables!
        let replies = db_conn.query_map("SELECT replies.trigger, comparator, ignore_case, reply, reply_type, reply_flag FROM replies ORDER BY rand()", |(trigger, comparator, ignore_case, reply, reply_type, reply_flag): (String, i8, i8, String, i8, i8)| {
            let comparator = match comparator {
                0 => Comparator::Contains,
                1 => Comparator::Equals,
                _ => Comparator::Undefined
            };

            let reply_type = match reply_type {
                0 => ReplyType::Text,
                3 => ReplyType::GifRandom,
                _ => ReplyType::Undefined
            };

            let ignore_case = if ignore_case == 1 { true } else { false };
            let reply_flag = if reply_flag == 1 { true } else { false };

            Reply {
                trigger,
                comparator,
                ignore_case,
                reply,
                reply_type,
                reply_flag
            }
        }).expect("Can't fetch replies from database");

        for reply in replies {
            let data = if reply.ignore_case { data.to_lowercase() } else { data.clone() };
            let should_reply = match reply.comparator {
                Comparator::Equals => data.eq(&reply.trigger),
                Comparator::Contains => data.contains(&reply.trigger),
                Comparator::Undefined => false,
            };

            if should_reply {
                match reply.reply_type {
                    ReplyType::Text  => {
                        if reply.reply_flag {
                            let _ = api.send(message.text_reply(reply.reply)).await;
                        } else {
                            let _ = api.send(message.chat.text(reply.reply)).await;
                        }
                    }
                    ReplyType::GifRandom => {
                        if reply.reply_flag {
                            let _ = api.send(message.document_reply(InputFileRef::new(reply.reply))).await;
                        } else {
                            let _ = api.send(message.chat.document(InputFileRef::new(reply.reply))).await;
                        }
                    },
                    ReplyType::Undefined => ()
                }
                return true;
            }
        }
    }

    false
}

pub async fn handle_tiktok(api: &Api, message: &Message) -> bool {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let re = Regex::new(r"tiktok.com").unwrap();
        if let Some(_caps) = re.captures(data) {
            let _ = api.send(message.delete()).await;
            return true;
        }
    }

    false
}

pub async fn handle_commands(api: &Api, mut _db_conn: PooledConn, ts_sender: Sender<TSCommand>, message: &Message) -> bool {

    if let MessageKind::Text { ref data, .. } = message.kind {
        if Regex::new(r"/kick [\w ]*").unwrap().is_match(data) {
            let re = Regex::new(r"/kick ([\w ]*)").unwrap();
            if let Some(target) = re.captures(data).unwrap().get(1) {
                let _ = ts_sender.send(TSCommand::UserKick(target.as_str().to_string(), "BrioschenBot".to_string(), String::new()));            // TODO Add ability to kick with message
                return true;
            }
        } else if Regex::new(r"/softkick [\w ]*").unwrap().is_match(data) {
            let re = Regex::new(r"/softkick ([\w ]*)").unwrap();
            if let Some(target) = re.captures(data).unwrap().get(1) {
                let _ = ts_sender.send(TSCommand::UserChannelKick(target.as_str().to_string(), "BrioschenBot".to_string(), String::new()));     // TODO Add ability to kick with message
                return true;
            }
        } else if Regex::new(r"/poke [\w ]*").unwrap().is_match(data) {
            let re = Regex::new(r"/poke ([\w ]*)").unwrap();
            if let Some(target) = re.captures(data).unwrap().get(1) {
                let _ = ts_sender.send(TSCommand::UserPoke(target.as_str().to_string(), "BrioschenBot".to_string(), String::new()));            // TODO Add ability to poke with message
                return true;
            }
        } else if Regex::new(r"/pokeall [\w ]*").unwrap().is_match(data) {
            let re = Regex::new(r"/pokeall ([\w ]*)").unwrap();
            if let Some(msg) = re.captures(data).unwrap().get(1) {
                let _ = ts_sender.send(TSCommand::ServerPokeAll("BrioschenBot".to_string(), msg.as_str().to_string()));
                return true;
            }
        } else if Regex::new(r"/hameln").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("http://webcam.hameln.de/CGIProxy.fcgi?cmd=snapPicture2&usr=WebCam&pwd=Hameln2017").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "hameln.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/weser").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("http://webcam2.hameln.de/CGIProxy.fcgi?cmd=snapPicture2&usr=WebCam&pwd=Hameln2017").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "weser.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/kluet").unwrap().is_match(data) || Regex::new(r"/klüt").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("http://webcam3.hameln.de:8080/cgi-bin/api.cgi?cmd=Snap&channel=0&rs=wuuPhkmUCeI9WG7C&user=web&password=hameln").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "kluet.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/schrobi").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("https://www.schrobenhausen.de/tools/webcams/feuerwehr/current.png").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "schrobi.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/neuburg").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("https://webcam.neuburg-donau.de/FoscamCamera_00626EA7EB36/snap/current.php").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "neuburg.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/pfaffen").unwrap().is_match(data) {
            if let Ok(response) = reqwest::get("https://video.pafunddu.de/webcam/paf_rathaus.jpg").await {
                if let Ok(data) = response.bytes().await {
                    let file = InputFileUpload::with_data(data, "pfaffen.jpg");
                    let _ = api.send(message.chat.photo(&file)).await;
                    return true;
                }
            }
            let _ = api.send(message.chat.text("Couldn't get webcam image"));
        } else if Regex::new(r"/rand (\d*)").unwrap().is_match(data) {
            let re = Regex::new(r"/rand (\d*)").unwrap();
            if let Some(max) = re.captures(data).unwrap().get(1) {
                let maximum = max.as_str().parse().unwrap_or(1000);
                let _ = api.send(message.text_reply(format!("{}", rand::random::<u32>() % maximum))).await;
                return true;
            }
        } else if Regex::new(r"/whothere").unwrap().is_match(data) {
            let (answer_sender, answer_receiver) : (Sender<String>, Receiver<String>) = mpsc::channel();

            if ts_sender.send(TSCommand::ServerUsersOnline(answer_sender)).is_ok() {
                let answer = answer_receiver.recv();
                if answer.is_ok() {
                    let _ = api.send(message.chat.text(answer.unwrap())).await;
                    return true;
                }
            }
        } else if Regex::new(r"/surprise").unwrap().is_match(data) {
            let (answer_sender, answer_receiver) : (Sender<String>, Receiver<String>) = mpsc::channel();

            if ts_sender.send(TSCommand::Surprise(answer_sender)).is_ok() {
                let answer = answer_receiver.recv();
                if answer.is_ok() {
                    let _ = api.send(message.chat.text(answer.unwrap())).await;
                    return true;
                }
            }
        }
    }

    false
}