// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::str::FromStr;
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
use serde::{Serialize, Deserialize};
use crate::data::{
    Resolution,
    OutputFormat,
    OutputSize,
};

///////////////////////////////////////////////////////////////////////////////
// DATA TYPES - OPT-PARAMETERS
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct OptParameters {
    size: OutputSize,
    format: OutputFormat,
}


impl TryFrom<http::Uri> for OptParameters {
    type Error = ();

    fn try_from(uri: http::Uri) -> Result<Self, Self::Error> {
        let query = uri
            .query()
            .unwrap_or(Default::default())
            .split("&")
            .filter_map(|param| -> Option<(String, String)> {
                let ix = param.find("=")?;
                let (left, right) = param.split_at(ix);
                let right = right.trim_start_matches("=");
                if left.is_empty() || right.is_empty() {
                    None
                } else {
                    Some((left.to_owned(), right.to_owned()))
                }
            })
            .collect::<HashMap<_, _>>();
        let size = query
            .get("size")
            .and_then(|x| OutputSize::from_str(x).ok())
            .unwrap_or_default();
        let format = query
            .get("format")
            .and_then(|x| OutputFormat::from_str(x).ok())
            .unwrap_or_default();
        Ok(OptParameters {
            size,
            format,
        })
    }
}



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
    let settings_result = OptParameters::try_from(req.uri().clone())
        .map_err(|_| format!("invalid url query parameters"));
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
        .and_then(|x| settings_result.map(|settings| {
            let xs: Vec<u8> = unimplemented!();
            xs
        }))
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
        .expect(&format!("bind to address {}", address))
        .run()
        .expect("imager http server");
}