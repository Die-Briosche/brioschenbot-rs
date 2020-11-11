use mysql::*;
use mysql::prelude::*;

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