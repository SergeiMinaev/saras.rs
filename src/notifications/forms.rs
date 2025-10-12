use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct PaginationForm {
	pub offset: Option<i32>,
	pub size: Option<i32>,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct MarkReadForm {
	#[validate(length(min = 1))]
	pub ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct MarkAllBeforeForm {
	#[validate(length(min = 10))]
	pub ts: String,
}
