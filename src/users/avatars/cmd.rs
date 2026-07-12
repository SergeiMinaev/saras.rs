use lpsql::Lpsql;
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::users::avatars::service::update_default_avatar;


pub async fn gen_default_avatars() {
    let pool = get_pool();
    let userdb = UserDb::new(pool);
    let mut offset = 0;
    let size = 20;
    loop {
        let page_users = userdb.page(offset, size, None, None, None).await;
        for user in &page_users {
            update_default_avatar(user.id.try_into().unwrap()).await;
        }
        if page_users.is_empty() {
            break;
        }
        offset += size;
    }
	println!("Done.");
}

pub async fn gen_default_avatars_missing() {
	let pool = get_pool();
	let mut offset = 0;
	let size = 100;
	let mut total = 0;
	loop {
		let rows = Lpsql::query("select id from users_users where default_avatar is null and name is not null order by id offset $1::INT limit $2::INT")
			.bind(offset)
			.bind(size)
			.fetch_all(&pool)
			.await;
		if rows.is_empty() {
			break;
		}
		for row in rows {
			if let Ok(id) = row.parse::<i32>() {
				let _ = update_default_avatar(id).await;
				total += 1;
			}
		}
		offset += size;
	}
	println!("Done. Updated {total} users.");
}
