use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::avl::AVLTreeSingleton;

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

pub async fn set(
    State(tree_singleton): State<Arc<AVLTreeSingleton>>,
    Json(request): Json<SetRequest>,
) -> Result<Json<Message>, StatusCode> {
    
    let tree = tree_singleton.get_instance();
    let mut tree = tree.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tree.set(&request.key, &request.value);
    
    Ok(Json(Message {
        value: Some(request.value),
        error: None,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct GetRequest {
    key: String,
}

pub async fn get(
    State(tree_singleton): State<Arc<AVLTreeSingleton>>,
    Json(request): Json<GetRequest>,
) -> Result<Json<Message>, StatusCode> {
    let tree = tree_singleton.get_instance();
    let tree = tree.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = tree.get(&request.key).map(|node| node.value.clone());

    let error = result.is_none().then(|| "Key not found".to_string());

    Ok(Json(Message {
        value: result,
        error,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRequest {
    key: String,
}

pub async fn delete(
    State(tree_singleton): State<Arc<AVLTreeSingleton>>,
    Json(request): Json<DeleteRequest>,
) -> Result<Json<Message>, StatusCode> {
    let tree = tree_singleton.get_instance();
    let mut tree = tree.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tree.unset(&request.key);

    Ok(Json(Message {
        value: None,
        error: None,
    }))
}
