use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::users::avatars::service::save_default_avatar;


pub async fn gen_default_avatars() {
    let pool = get_pool();
    let userdb = UserDb::new(pool);
    let mut offset = 0;
    let size = 20;
    loop {
        let page_users = userdb.page(offset, size).await;
        for user in &page_users {
            save_default_avatar(user.id.try_into().unwrap()).await;
        }
        if page_users.is_empty() {
            break;
        }
        offset += size;
    }
	println!("Done.");
}
