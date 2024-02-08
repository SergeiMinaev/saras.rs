pub use miarh_saras_http::Request;
use crate::users::users;


pub trait RequestTools {
    fn get_user(&self) -> Option<users::models::User>;
    fn is_su(&self) -> bool;
}

impl RequestTools for Request {
    fn get_user(&self) -> Option<users::models::User> {
        if self.session_id != "".to_string() {
            return users::models::User::by_session_id(&self.session_id);
        } else { return None }
    }
    fn is_su(&self) -> bool {
        match self.get_user() {
            None => return false,
            Some(u) => return u.is_superuser,
        }
    }
}
