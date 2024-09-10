use serde::Deserialize;
use serde_json::{ json };
use validator::{ Validate };
use crate::posts::posts::models::Post;
use crate::posts::posts::db::PostDb;
use crate::http::{ Request };
use crate::posts::posts::forms::{ CreatePostForm };
use crate::validation::ValidationErrors;
use crate::posts::posts::forms::PostForm;
use crate::errors::Error;


pub fn create_post(post_form: PostForm) -> Result<Post, ValidationErrors> {
	Ok(PostDb::create_and_get(post_form).unwrap())
}


pub async fn update_post(id: i32, post_form: PostForm) -> Result<Post, Error> {
	Ok(PostDb::update_and_get(id, post_form).await.unwrap())
}


pub fn delete_post(id: i32) -> bool {
	PostDb::delete(id)
}
