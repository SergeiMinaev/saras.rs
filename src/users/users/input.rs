use serde::Deserialize;
use serde_json::{ json };
use validator::{ Validate };
use crate::users::users::models::User;
use crate::http::{ Request };



#[derive(Debug, Validate, Deserialize, schemars::JsonSchema)]
pub struct UserInput {
	#[validate(length(min = 5))]
	pub email: String,
	#[validate(length(min = 10))]
	pub pwd: Option<String>,
	pub is_superuser: bool,
	pub avatar: Option<String>,
}

impl UserInput {
	pub fn new(data: &str) -> Result<Self, serde_json::Value> {
		match serde_json::from_str::<Self>(data) {
			Err(e) => return Err(err_before_deser(e)),
			Ok(v) => Ok(v),
		}
	}
	pub async fn validatee(&mut self, _req: &Request) -> Result<serde_json::Value, serde_json::Value> {
		//let ok = validate(self).is_ok();
		validate_mut(self)

	}
	pub fn upload_file(&mut self, req: i32) -> i32 {
	  req
	}
	pub async fn write(id: Option<i32>, req: &Request) -> Result<User, serde_json::Value> {
	  match UserInput::new(&req.body_string) {
		Err(e) => return Err(e),
		Ok(mut input) => {
		  match input.validatee(req).await {
			Err(e) => return Err(e),
			Ok(_) => {
			  if id.is_some() {
				match User::update_and_get(id.unwrap(), input).await {
				  None => return Err(json!({"msg": "Не удалось изменить пользователя."})),
				  Some(v) => return Ok(v)
				}
			  } else {
				match User::create_and_get(input) {
				  None => return Err(json!({"msg": "Не удалось создать пользователя."})),
				  Some(v) => return Ok(v)
				}
			  }
			}
		  }
		}
	  }
	}
	pub async fn create(req: &Request) -> Result<User, serde_json::Value> {
	  UserInput::write(None, req).await
	}
	pub async fn update(req: &Request) -> Result<User, serde_json::Value> {
	  let id: i32 = req.route.get("id").unwrap().parse().unwrap();
	  println!("update model {id}");
	  UserInput::write(Some(id), req).await
	}
}

pub fn validate_mut<T: Validate>(obj: &mut T) -> Result<serde_json::Value, serde_json::Value> {
	match obj.validate() {
		Err(e) => {
			//for e in e.errors() {
			//	  println!("EEE: {e:?}");
			//}
			let r: serde_json::Value = serde_json::to_value(&e).unwrap();
			let r = json!({ "field": r });
			return Err(r)
		},
		Ok(_) => {
			return Ok(json!({"ok": "2"}))
		},
	}
}

pub fn validate<T: Validate>(obj: T) -> Result<serde_json::Value, serde_json::Value> {
	match obj.validate() {
		Err(e) => {
			//for e in e.errors() {
			//	  println!("EEE: {e:?}");
			//}
			let r: serde_json::Value = serde_json::to_value(&e).unwrap();
			let r = json!({ "field": r });
			return Err(r)
		},
		Ok(_) => {
			return Ok(json!({"ok": "2"}))
		},
	}
}

pub fn err_before_deser(e: serde_json::Error) -> serde_json::Value {
	let s = e.to_string();
	println!("Cant make struct: {:?}", s);
	let code = s.split("`").nth(0).unwrap().trim().replace(" ", "_");
	let field = s.split("`").nth(1).unwrap_or_default();
	json!({ "type": code, "field": field, "raw": s })
}
