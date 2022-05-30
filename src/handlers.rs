use crate::MerkleTree;
use actix_web::body::BoxBody;
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::ContentType;

use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, Result};
use serde::Serialize;

use std::collections::HashMap;

#[derive(Serialize)]
struct HashesResponse {
    hash: String,
    pieces: usize,
}

#[derive(Serialize)]
pub struct PiecesResponse {
    content: String,
    proof: Vec<String>,
}

impl Responder for HashesResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for PiecesResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

pub async fn get_hashes(trees: web::Data<HashMap<String, MerkleTree>>) -> impl Responder {
    let current_tree = trees.values().next().unwrap();
    HashesResponse {
        hash: hex::encode(&current_tree.nodes[1]),
        pieces: current_tree.total_non_empty_pieces,
    }
}

pub async fn get_piece(
    trees: web::Data<HashMap<String, MerkleTree>>,
    path: web::Path<(String, String)>,
) -> Result<PiecesResponse, Error> {
    let (hash_index, piece_index_str) = path.into_inner();
    let piece_index_num = piece_index_str.parse::<usize>().unwrap();
    if !trees.contains_key(&*hash_index) {
        return Err(ErrorBadRequest("No files available for hash requested"));
    }

    let current_tree = trees.get(&*hash_index).unwrap();

    if !current_tree.piece_data.contains_key(&piece_index_num) {
        return Err(ErrorBadRequest("Invalid piece requested"));
    }

    let piece_data = current_tree.piece_data.get(&piece_index_num).unwrap();

    let response = PiecesResponse {
        content: piece_data.to_owned(),
        proof: current_tree.proof(piece_index_num).unwrap(),
    };

    return Ok(response);
}
