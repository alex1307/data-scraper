
use serde::{Deserialize, Serialize};

use super::traits::{Identity, Header};



#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IDList {
	pub id: String,
}

impl IDList {
	pub fn new(id: String) -> Self {
		Self { id }
	}
}

impl Identity for IDList {
	fn get_id(&self) -> String {
		self.id.clone()
	}
}

impl Header for IDList {
	fn header() -> Vec<&'static str> {
		vec!["id"]
	}
}