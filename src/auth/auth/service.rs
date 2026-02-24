use crate::http::{ Request, Resp, forbidden };
use crate::request::RequestTools;
use futures_lite::{ Future };
use crate::db::get_pool;
use crate::users::users::db::UserDb;
use crate::auth::auth::db::AuthDb;
use crate::users::users::forms::UserForm;
use crate::auth::auth::models::Session;
use log::debug;


pub async fn check_su<F, Fut>(req: Request,
    f: F
) -> Resp
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Resp>
{
    if !req.is_su().await { return forbidden() }
    f(req).await
}

pub async fn login_by_email(email: &str) -> Option<Session> {
	let pool = get_pool();
	let userdb = UserDb::new(pool.clone());
	let authdb = AuthDb::new(pool.clone());
	match userdb.by_email(email).await {
		None => {
			let form = UserForm {
				email: email.to_string(),
				pwd: None, // без пароля - вход через OAuth
				is_superuser: Some(false),
				avatar: None,
				name: None,
			};
			debug!("Нужно создать: {form:?}");
			let user = userdb.create_and_get(form).await?;
			debug!("444");
			return authdb.add_session(Some(user.id)).await
		},
		Some(u) => {
			debug!("Добавляю сессию к имеющемуся");
			debug!("555");
			return authdb.add_session(Some(u.id)).await
		},
	};
}
