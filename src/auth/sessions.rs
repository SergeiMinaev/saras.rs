use serde::{ Serialize,Deserialize };
use lpsql::QueryParam as qp;


#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: String,
    pub expires: String,
    pub user_id: u32,
}

impl Session {
    pub fn by_id(id: String) -> Option<Session> {
        let prms: Vec<qp> = vec![
            qp::String(id)
        ];
        let query = "select row_to_json(data) from (\
            select id, expires, user_id from auth_sessions where id = $1::BYTEA \
        ) data";
        match lpsql::get_one(query, prms) {
            None => None::<Session>,
            Some(v) => {
                return serde_json::from_str(&v).unwrap();
            }
        };
        None
    }
}
