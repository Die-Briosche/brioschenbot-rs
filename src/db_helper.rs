use mysql::*;
use mysql::prelude::*;
use telegram_bot::{Message, MessageText};

pub fn get_alias_from_telegram_id(mut db_conn: PooledConn, t_id: &String, name: Option<(String, String, String)>) -> String {
    let aliasses = db_conn.query_map(format!("SELECT alias FROM users WHERE t_uid = {}", t_id), |alias: String| {     // Usually you wouldn't use format! for your query for SQL injection reasons
                                                                                                                                            // t_id isn't something a user could supply, so it should be fine here
        alias
    }).expect("Can't fetch users from database");

    if aliasses.len() != 0 {
        aliasses[0].to_string()
    } else {
        if name.is_some() {
            let name_u = name.unwrap();
            db_conn.exec_drop("INSERT INTO users (id, t_uid, first_name, last_name, ts_uid, ts_uid2, alias, ts_name, ip, timestamp, comment) VALUES (0, :t_uid, :fname, :lname, '', '', :alias, '', '', NOW(), '')", params!{
                "t_uid" => t_id,
                "fname" => name_u.0,
                "lname" => name_u.1,
                "alias" => name_u.2
            }).unwrap();
        }
        "".to_string()
    }
}

pub fn get_tsname_from_userid(db_conn: &mut PooledConn, id: &String) -> String {
    let tsnames = db_conn.query_map(format!("SELECT ts_name FROM users WHERE id = {}", id), |tsname: String| {        // Usually you wouldn't use format! for your query for SQL injection reasons
                                                                                                                                        // id isn't something a user could supply, so it should be fine here
        tsname
    }).expect("Can't fetch users from database");

    if tsnames.len() != 0 {
        return tsnames[0].to_string();
    }
    String::new()
}

pub fn log_message(db_conn: &mut PooledConn, msg: Message) {
    let _ = db_conn.exec_drop("INSERT INTO messages (userid, message, chatid, messageid, timestamp) VALUES (:userid, :message, :chatid, :messageid, :timestamp)", params!{
        "userid" =>  i64::from(msg.from.id),
        "message" => msg.text().unwrap_or("".to_string()),
        "chatid" => i64::from(msg.chat.id()),
        "messageid" => i64::from(msg.id),
        "timestamp" => msg.date,
    });
}