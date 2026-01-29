use crate::db::get_pool;
use crate::users::profile::db::ProfileDb;
use crate::users::avatars::service::update_default_avatar;



pub async fn set_name(user_id: u32, name: &str) -> bool {
	let pool = get_pool();
	let profiledb = ProfileDb::new(pool);
	let ok = profiledb.set_name(user_id, name).await;
	if ok {
		let user_id_i32: i32 = user_id.try_into().unwrap_or_default();
		if user_id_i32 > 0 {
			let _ = update_default_avatar(user_id_i32).await;
		}
	}
	ok
}
