// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use futures::{Future, Stream};
use actix_web::{
    web,
    App,
    Responder,
    HttpServer,
    HttpRequest,
    HttpResponse,
};
use actix_web::http::StatusCode;




///////////////////////////////////////////////////////////////////////////////
// HTTP ROUTES
///////////////////////////////////////////////////////////////////////////////

fn index(request: HttpRequest) -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION");
    HttpResponse::Ok().body(format!(
        "imager server, version '{version}'.",
        version=version,
    ))
}

fn opt_route(
    req: HttpRequest,
    body: web::Payload,
) -> impl Future<Item = HttpResponse, Error = actix_web::error::Error> {
    let settings_result: Result<(), ()> = unimplemented!();
    let result = body
        .map_err(|e| {
            eprintln!("payload error: {:?}", e);
            ()
        })
        .fold::<_, actix_web::web::BytesMut, _>(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok(body)
        })
        .map_err(|e| format!("http request payload issue"))
        .map(|x| x.to_vec())
        .and_then(|x| match settings_result {
            Ok(settings) => {
                let xs: Vec<u8> = unimplemented!();
                Ok(xs)
            },
            Err(()) => Err(String::from("invalid url query parameters"))
        })
        .map_err(|e| {
            let x = HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("{}", e));
            let x: actix_web::error::Error = From::from(x);
            x
        })
        .and_then(|x| {
            HttpResponse::Ok()
                .content_type("image/jpeg")
                .body(x)
        });
    result

}


///////////////////////////////////////////////////////////////////////////////
// EXTERNAL API
///////////////////////////////////////////////////////////////////////////////

pub fn run(address: &str) {
    println!("running server on: {}", address);
    let server = || App::new().service(
        web::resource("/").route(web::get().to(index))
    );
    HttpServer::new(server)
        .bind(address)
        .expect(&format!("unable to bind to address {}", address))
        .run()
        .expect("run server failed");
}