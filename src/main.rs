use actix::prelude::*; // Actor traits for WebSocket handling.
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Error, HttpRequest};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use reqwest;
use dotenv::dotenv;
use std::env;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use std::time::{Duration as StdDuration, Instant};

/// Claims for JWT token.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Request for fetching info: a cryptocurrency symbol and authentication token.
#[derive(Debug, Deserialize)]
struct InfoRequest {
    symbol: String,
    token: Option<String>,
}

/// Request payload for login.
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Response payload for a successful login.
#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

/// Secret key for JWT signing (in production, load securely).
static JWT_SECRET: &str = "secret";

/// Validate a JWT token using the secret.
fn validate_token(token: &str) -> bool {
    let decoding_key = DecodingKey::from_secret(JWT_SECRET.as_ref());
    let validation = Validation::default();
    decode::<Claims>(token, &decoding_key, &validation).is_ok()
}

/// POST /login  
/// Expects JSON payload {"username": "...", "password": "..."}.  
/// For valid credentials (dummy check: username "admin", password "password"),
/// returns a JWT token.
#[post("/login")]
async fn login(form: web::Json<LoginRequest>) -> impl Responder {
    if form.username == "admin" && form.password == "password" {
        let expiration = Utc::now() + Duration::hours(24);
        let claims = Claims {
            sub: form.username.clone(),
            exp: expiration.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_ref())
        ).expect("Failed to encode token");
        HttpResponse::Ok().json(LoginResponse { token })
    } else {
        HttpResponse::Unauthorized().body("Invalid credentials")
    }
}

/// GET /news?symbol=XYZ&token=...
/// Verifies the JWT token and then fetches raw JSON from CoinMarketCap's Info API
/// for the given cryptocurrency symbol and forwards it to the client.
#[get("/news")]
async fn get_info(query: web::Query<InfoRequest>) -> impl Responder {
    // Require token for authentication.
    if let Some(ref token) = query.token {
        if !validate_token(token) {
            return HttpResponse::Unauthorized().body("Invalid token");
        }
    } else {
        return HttpResponse::Unauthorized().body("Token required");
    }

    let symbol = &query.symbol;
    match fetch_raw_info(symbol).await {
        Ok(json_value) => HttpResponse::Ok().json(json_value),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error fetching info: {:?}", e)),
    }
}

/// Fetches raw JSON from the CoinMarketCap Info API for the given symbol.
/// API Documentation: https://coinmarketcap.com/api/documentation/v1
async fn fetch_raw_info(symbol: &str) -> Result<serde_json::Value, reqwest::Error> {
    let api_key = env::var("COINMARKETCAP_API_KEY").unwrap_or_else(|_| "demo".to_string());
    let url = format!("https://pro-api.coinmarketcap.com/v2/cryptocurrency/info?symbol={}", symbol);

    let client = reqwest::Client::new();
    let res = client.get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .send()
        .await?;
    let json_value = res.json::<serde_json::Value>().await?;
    Ok(json_value)
}

/// WebSocket session actor for real-time updates.
struct MyWs {
    hb: Instant,
}

impl MyWs {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }
    
    /// Starts a heartbeat process that pings the client every 5 seconds.
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(StdDuration::new(5, 0), |act, ctx| {
            if Instant::now().duration_since(act.hb) > StdDuration::new(10, 0) {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            Ok(ws::Message::Ping(msg)) => { 
                self.hb = Instant::now(); 
                ctx.pong(&msg); 
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(text)) => ctx.text(format!("Echo: {}", text)),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => { ctx.close(reason); ctx.stop(); },
            _ => (),
        }
    }
}

/// Handler for establishing a WebSocket connection at /ws.
/// **Note:** Use a WebSocket client (e.g. wscat) instead of a browser, or a 400 error will be shown.
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWs::new(), &req, stream)
}

/// GET /
/// Displays a basic HTML form for user interaction.
#[get("/")]
async fn index() -> impl Responder {
    let html = r#"
    <!DOCTYPE html>
    <html>
      <head>
        <title>Cryptocurrency Info Aggregator</title>
      </head>
      <body>
        <h1>Cryptocurrency Info Aggregator</h1>
        <form action="/news" method="get">
          <label for="symbol">Enter cryptocurrency symbol:</label>
          <input type="text" id="symbol" name="symbol" placeholder="e.g., BTC" required>
          <br><br>
          <label for="token">JWT Token:</label>
          <input type="text" id="token" name="token" placeholder="Enter token" required>
          <br><br>
          <button type="submit">Get Info</button>
        </form>
        <p>
          To login, send a POST request to <code>/login</code> with JSON payload:
          <br>{"username": "admin", "password": "password"}
        </p>
        <p>For real-time updates, connect to the WebSocket endpoint at <code>/ws</code>
           using a WebSocket client (e.g., <code>wscat -c ws://127.0.0.1:8080/ws</code>).</p>
      </body>
    </html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// Main entry point: load environment variables, build and run the HTTP server.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("Server running at http://127.0.0.1:8080/");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(login)
            .service(get_info)
            .route("/ws", web::get().to(ws_index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
