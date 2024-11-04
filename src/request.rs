pub use miarh_saras_http::Request;
use crate::users::users;


pub trait RequestTools {
    fn get_user(&self) -> impl std::future::Future<Output = Option<users::models::User>> + Send;
    fn is_su(&self) -> impl std::future::Future<Output = bool> + Send;
}

impl RequestTools for Request {
    async fn get_user(&self) -> Option<users::models::User> {
        if self.session_id != "".to_string() {
            return users::models::User::by_session_id(&self.session_id).await;
        } else { return None }
    }
    async fn is_su(&self) -> bool {
        match self.get_user().await {
            None => return false,
            Some(u) => return u.is_superuser,
        }
    }
}
