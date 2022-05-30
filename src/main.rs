use crate::file_utils::{pad_leaf_layer, split_file_to_chunks};
use crate::merkle_tree::MerkleTree;

use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};

mod file_utils;
mod handlers;
mod merkle_tree;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut trees: HashMap<String, MerkleTree> = HashMap::new();
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        Error::new(ErrorKind::Other, "Invalid number of arguments");
    }

    let tree = MerkleTree::new(&args[1]);

    trees.insert(hex::encode(&tree.nodes[1]), tree);

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(trees.clone()))
            .route("/hashes", web::get().to(handlers::get_hashes))
            .route(
                "/piece/{hashId}/{pieceIndex}",
                web::get().to(handlers::get_piece),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::{get_hashes, get_piece};
    use actix_web::{
        http::{header::ContentType},
        test, web, App,
    };

    #[actix_web::test]
    async fn test_correct_hashes() {
        let tree = MerkleTree::new("test_data/icons_rgb_circle.png");
        let mut trees: HashMap<String, MerkleTree> = HashMap::new();
        trees.insert(hex::encode(&tree.nodes[1]), tree);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(trees.clone()))
                .route("/", web::get().to(get_hashes)),
        )
        .await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_pieces_wrong_hash() {
        let tree = MerkleTree::new("test_data/icons_rgb_circle.png");
        let mut trees: HashMap<String, MerkleTree> = HashMap::new();
        trees.insert(hex::encode(&tree.nodes[1]), tree);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(trees.clone()))
                .route("/piece/{hashId}/{pieceIndex}", web::get().to(get_piece)),
        )
        .await;
        let req = test::TestRequest::with_uri("/piece/asdf/4")
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(!resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_pieces_correct() {
        let tree = MerkleTree::new("test_data/icons_rgb_circle.png");
        let mut trees: HashMap<String, MerkleTree> = HashMap::new();
        trees.insert(hex::encode(&tree.nodes[1]), tree);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(trees.clone()))
                .route("/piece/{hashId}/{pieceIndex}", web::get().to(get_piece)),
        )
        .await;
        let req = test::TestRequest::with_uri(
            "/piece/9b39e1edb4858f7a3424d5a3d0c4579332640e58e101c29f99314a12329fc60b/4",
        )
        .insert_header(ContentType::plaintext())
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_pieces_wrong_piece_number() {
        let tree = MerkleTree::new("test_data/icons_rgb_circle.png");
        let mut trees: HashMap<String, MerkleTree> = HashMap::new();
        trees.insert(hex::encode(&tree.nodes[1]), tree);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(trees.clone()))
                .route("/piece/{hashId}/{pieceIndex}", web::get().to(get_piece)),
        )
        .await;
        let req = test::TestRequest::with_uri(
            "/piece/9b39e1edb4858f7a3424d5a3d0c4579332640e58e101c29f99314a12329fc60b/43232",
        )
        .insert_header(ContentType::plaintext())
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(!resp.status().is_success());
    }
}
