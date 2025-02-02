use serde::Deserialize;
use validator::{Validate};



#[derive(Debug, Validate, Deserialize, Clone)]
pub struct SetNameForm {
	pub name: String,
}
