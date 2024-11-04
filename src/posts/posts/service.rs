use crate::posts::posts::models::Post;
use crate::posts::posts::db::PostDb;
use crate::validation::ValidationErrors;
use crate::posts::posts::forms::PostForm;
use crate::errors::Error;
use crate::db::get_pool;


pub async fn create_post(post_form: PostForm) -> Result<Post, ValidationErrors> {
	let pool = get_pool();
	let postdb = PostDb::new(pool.clone());
	Ok(postdb.create_and_get(post_form).await.unwrap())
}


pub async fn update_post(id: i32, post_form: PostForm) -> Result<Post, Error> {
	let pool = get_pool();
	let postdb = PostDb::new(pool.clone());
	Ok(postdb.update_and_get(id, post_form).await.unwrap())
}


pub async fn delete_post(id: i32) -> bool {
	let pool = get_pool();
	let postdb = PostDb::new(pool.clone());
	postdb.delete(id).await
}
