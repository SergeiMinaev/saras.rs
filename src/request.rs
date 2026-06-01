pub use miarh_saras_http::Request;
use crate::users::users;
use crate::users::users::db::UserDb;
use crate::db::get_pool;


pub trait RequestTools {
	fn get_user(&self) -> impl std::future::Future<Output = Option<users::models::User>> + Send;
	fn is_su(&self) -> impl std::future::Future<Output = bool> + Send;
}

impl RequestTools for Request {
	async fn get_user(&self) -> Option<users::models::User> {
		if self.session_id != "".to_string() {
			let pool = get_pool();
			let userdb = UserDb::new(pool.clone());
			let user = userdb.by_session_id(&self.session_id).await;
			if let Some(ref u) = user {
				let user_id = u.id;
				let pool2 = pool.clone();
				smol::spawn(async move {
					UserDb::new(pool2).touch_last_visit(user_id).await;
				}).detach();
			}
			return user;
		} else { return None }
	}
	async fn is_su(&self) -> bool {
		match self.get_user().await {
			None => return false,
			Some(u) => return u.is_superuser,
		}
	}
}
