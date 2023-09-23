use sov_modules_api::Context;

use super::Transfer;

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
pub struct Response {
    pub value: Option<u32>,
}

impl<C> Transfer<C> where C: Context {}
