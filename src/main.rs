#[macro_use]
extern crate diesel;
extern crate juniper;

use std::sync::Arc;

use actix_web::{web, Error, App, HttpResponse, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod graphql_schema;
mod schema;

use crate::graphql_schema::{create_schema, Schema};

fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
        .map_err(Error::from)
        .and_then(|user| {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user))
        })
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("https://canduma.rust.localhost:8088/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn main() {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    let schema = std::sync::Arc::new(create_schema());

    builder
        .set_private_key_file("certs/rust.localhost.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("certs/rust.localhost.crt").unwrap();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
        .bind_ssl("127.0.0.1:8088", builder)
        .unwrap()
        .run()
        .unwrap();
}