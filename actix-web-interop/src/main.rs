//! This example creates an actix-web server with two routes:
//!
//! * /{id}/{name}/
//!   For example: /42/John/
//!
//! * /visitor_count
//!
//! Each time the first route is visited the handler function sends a
//! message to the `VisitCounter` actor. The second route returns the
//! visit count.

use act_zero::runtimes::default::spawn_actor;
use act_zero::*;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

struct VisitCounter {
    visit_count: i32,
}

impl Actor for VisitCounter {}

impl VisitCounter {
    /// Increments and returns visitor count.
    ///
    /// Because this message receiver takes an `&mut self` argument only
    /// only one message may be handled at a time.
    async fn visit(&mut self) -> ActorResult<i32> {
        self.visit_count += 1;
        Produces::ok(self.visit_count)
    }

    /// Returns the unmodified visitor count
    ///
    /// Because this message receiver takes an `&self` argument
    /// multiple message may be handled concurrently.
    async fn get_count(&self) -> ActorResult<i32> {
        Produces::ok(self.visit_count)
    }
}

/// Returns a message with the path fields and the visitor count.
/// The mutable self reference limits concurrency.
///
/// # Arguments:
///
/// * `req` - We include the HttpRequest object so we can get the actor_ref with app_data.
///
#[get("/{id}/{name}/")]
async fn index(
    web::Path((id, name)): web::Path<(u32, String)>,
    req: HttpRequest,
) -> impl Responder {
    let actor_ref = req
        .app_data::<Addr<VisitCounter>>()
        .expect("failed to get visitor count actor_ref!");

    let count = match call!(actor_ref.visit()).await {
        Ok(count) => count,
        Err(_error) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().body(format!("Hello {}! id:{} visitors:{}", name, id, count))
}

/// Returns the current visitor count.
/// The const self reference allows for concurrency.
#[get("/visitor_count")]
async fn visitor_count(req: HttpRequest) -> impl Responder {
    let actor_ref = req
        .app_data::<Addr<VisitCounter>>()
        .expect("failed to get visitor count actor_ref!");

    let count = match call!(actor_ref.get_count()).await {
        Ok(count) => count,
        Err(_error) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(count)
}

// The `#[actix_web::main]` attribute initializes the tokio executor.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Because we included act-zero with the "default-tokio" feature
    // it will automatically use that executor to run our actors when
    // we spawn it with `spawn_actor`.
    let actor_ref = spawn_actor(VisitCounter { visit_count: 0 });
    HttpServer::new(move || {
        App::new()
            // Calling `app_data` with our actor_ref allows it to be
            // accessed in route handlers.
            .app_data(actor_ref.clone())
            .service(index)
            .service(visitor_count)
    })
    .bind("0.0.0.0:9999")?
    .run()
    .await
}
