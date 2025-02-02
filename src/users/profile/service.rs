use crate::db::get_pool;
use crate::users::profile::db::ProfileDb;



pub async fn set_name(user_id: u32, name: &str) -> bool {
	let pool = get_pool();
	let profiledb = ProfileDb::new(pool);
	profiledb.set_name(user_id, name).await
}
