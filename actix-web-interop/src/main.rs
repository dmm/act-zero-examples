use act_zero::*;
use actix_web::{get, web, App, HttpServer, Responder};
use tokio::runtime::Handle;

struct VisitCounter {
    visit_count: i32,
}

impl Actor for VisitCounter {}

impl VisitCounter {
    async fn visit(&mut self) -> ActorResult<i32> {
        self.visit_count += 1;
        Produces::ok(self.visit_count)
    }
}

#[get("/{id}/{name}/index.html")]
async fn index(
    actor_ref: Addr<VisitCounter>,
    web::Path((id, name)): web::Path<(u32, String)>
) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let handle = Handle::current();
    let actor_ref = Addr::new(
        &handle,
        VisitCounter {
            visit_count: 0,
        }
    );
    HttpServer::new(|| App::new()
                    .app_data(actor_ref)
                    .service(index))
        .bind("127.0.0.1:9999")?
        .run()
        .await
}
