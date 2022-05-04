use crate::db::{init_db, upsert};
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct User {
  #[serde(rename = "_id")]
  pub id: String,

  #[serde(default)]
  pub authorized: bool,

  #[serde(default)]
  pub pinned_message_id: i32,
}

async fn collection() -> anyhow::Result<Collection<User>> {
  let db = init_db(crate::CONFIG.clone()).await.unwrap();

  Ok(db.collection("users"))
}

// pub async fn get_user(user_id: i64) -> anyhow::Result<User> {
//   let users = collection().await?;
//   let user = users.find_one(bson::doc!{ "_id": user_id.to_string() }, None).await?;
//
//   match user {
//     Some(user) => Ok(user),
//     None => Ok(User { id: user_id.to_string(), authorized: false })
//   }
// }

pub async fn get_users() -> anyhow::Result<Vec<User>> {
  let users = collection().await?;
  let users: Vec<User> = users
    .find(bson::doc! { "authorized": true }, None)
    .await?
    .try_collect()
    .await?;

  Ok(users)
}

pub async fn set_authorized(user_id: i64, state: bool) -> anyhow::Result<()> {
  let users = collection().await?;
  let user = User {
    id: user_id.to_string(),
    authorized: state,
    pinned_message_id: 0
  };
  users
    .find_one_and_update(
      bson::doc! { "_id": user_id.to_string() },
      bson::doc! { "$set": bson::to_document(&user)? },
      upsert(),
    )
    .await?;

  log::info!("Authorized {}", user_id);

  Ok(())
}


pub async fn set_pinned(user: &mut User, pinned_message_id: i32) -> anyhow::Result<()> {
  let users = collection().await?;

  user.pinned_message_id = pinned_message_id;

  users
    .find_one_and_update(
      bson::doc! { "_id": user.id.to_string() },
      bson::doc! { "$set": bson::to_document(&user)? },
      upsert(),
    )
    .await?;

  Ok(())
}
