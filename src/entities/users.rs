use sqlx::{Decode, Encode};

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub surname: String,
}
