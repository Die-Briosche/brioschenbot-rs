use telegram_bot::{Api, MessageText, CanReplySendMessage, MessageKind, CanSendMessage, Message};
use mysql::PooledConn;
use mysql::prelude::Queryable;
use crate::reply::{Comparator, Reply, ReplyType};


pub async fn handle_replies(api: &Api, mut db_conn: PooledConn, message: &Message) -> bool {
    if let MessageKind::Text { ref data, .. } = message.kind {

        let replies = db_conn.query_map("SELECT replies.trigger, comparator, ignore_case, reply, reply_type, reply_flag FROM replies", |(trigger, comparator, ignore_case, reply, reply_type, reply_flag): (String, i8, i8, String, i8, i8)| {
            let comparator = match comparator {
                0 => Comparator::Contains,
                1 => Comparator::Equals,
                _ => Comparator::Undefined
            };

            let reply_type = match reply_type {
                0 => ReplyType::Text,
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
            println!("Testing {} vs {}", reply.trigger, data);
            let data = if reply.ignore_case { data.to_lowercase() } else { data.clone() };
            let should_reply = match reply.comparator {
                Comparator::Equals => data.eq(&reply.trigger),
                Comparator::Contains => data.contains(&reply.trigger),
                Comparator::Undefined => false,
            };

            if should_reply {
                if let ReplyType::Text = reply.reply_type {
                    if reply.reply_flag {
                        api.send(message.text_reply(reply.reply)).await.unwrap();
                    } else {
                        api.send(message.chat.text(reply.reply)).await.unwrap();
                    }
                }

                return true;
            }
        }
    }

    false
}