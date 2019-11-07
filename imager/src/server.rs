use actix_web::{
    web,
    App,
    Responder,
    HttpServer,
    HttpRequest,
    HttpResponse,
};
use actix_web::http::StatusCode;


fn index(request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .body("hello world")
}


///////////////////////////////////////////////////////////////////////////////
// EXTERNAL API
///////////////////////////////////////////////////////////////////////////////

pub fn run(address: &str) {
    let server = || App::new().service(
        web::resource("/").route(web::get().to(index))
    );
    HttpServer::new(server)
        .bind(address)
        .expect(&format!("unable to bind to address {}", address))
        .run()
        .expect("run server failed");
}