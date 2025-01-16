use serde::{ Serialize,Deserialize };


#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
	pub id: String,
	pub expires: String,
	pub user_id: u32,
}
