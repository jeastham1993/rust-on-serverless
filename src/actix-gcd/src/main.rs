use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    server
        .bind("127.0.0.1:3000")?
        .run()
        .await
}

async fn post_gcd(form: web::Form<GcdParameters>) -> impl Responder {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("<h1>Computing the GCD with 0 is boring, pick a positive integer</h1>")
    }

    let value = gcd(form.m , form.n);

    let response = format!("<h1>{} and {} have a GCD of {}</h1>", form.m, form.n, value);
    
    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}

async fn get_index() -> impl Responder {
    HttpResponse::Ok()
      .content_type("text/html")
      .body(
        r#"
            <title>GCD Calculator</title>
            <form action="/gcd" method="post">
                <input type="text" name="n" />
                <input type="text" name="m" />
                <button type="submit">Compute GCD</button>
            </form>
        "#,
      )
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);

    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64
}

