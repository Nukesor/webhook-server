use ::actix::prelude::*;
use std::collections::HashMap;

#[derive(Message)]
pub struct NewTask {
    pub id: String,
    pub parameters: HashMap<String, String>,
}
