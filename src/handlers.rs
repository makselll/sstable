use crate::avl::AVLTreeSingleton;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Message {
    value: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SetRequest {
    key: String,
    value: String,
}

#[post("/set", format = "application/json", data = "<request>")]
pub fn set(
    request: Json<SetRequest>,
    tree_singleton: &State<AVLTreeSingleton>,
) -> Result<Json<Message>, Status> {
    let tree = tree_singleton.get_instance();
    let mut tree = tree.write().map_err(|_| Status::InternalServerError)?;

    tree.set(&request.key, &request.value);

    Ok(Json(Message {
        value: Some(request.value.clone()),
        error: None,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct GetRequest {
    key: String,
}

#[post("/get", format = "application/json", data = "<request>")]
pub fn get(
    request: Json<GetRequest>,
    tree_singleton: &State<AVLTreeSingleton>,
) -> Result<Json<Message>, Status> {
    let tree = tree_singleton.get_instance();
    let tree = tree.read().map_err(|_| Status::InternalServerError)?;

    let result = tree.get(&request.key);

    Ok(Json(Message {
        value: result.map(|node| node.value.clone()),
        error: result.map_or_else(|| Some("Key not found".to_string()), |_| None),
    }))
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRequest {
    key: String,
}

#[delete("/delete", format = "application/json", data = "<request>")]
pub fn delete(
    request: Json<DeleteRequest>,
    tree_singleton: &State<AVLTreeSingleton>,
) -> Result<Json<Message>, Status> {
    let tree = tree_singleton.get_instance();
    let mut tree = tree.write().map_err(|_| Status::InternalServerError)?;

    tree.unset(&request.key);

    Ok(Json(Message {
        value: None,
        error: None,
    }))
}
