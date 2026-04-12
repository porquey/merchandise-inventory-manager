use axum::{
    Form, Router, extract::State, response::{Redirect, IntoResponse}, routing::{get, post}
};
use axum::body::Body;
use tokio::net::TcpListener;
use std::{net::SocketAddr};
use time::{Duration, OffsetDateTime};

use tower_http::services::ServeDir;
use tokio;
use askama::Template;
use sqlx::SqlitePool;
use sqlx::FromRow;
use serde::Deserialize;
use axum::extract::Query;
use std::collections::HashMap;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum::{
    http::Request,
    middleware::Next,
};

async fn admin_auth(req: Request<Body>, next: Next) -> impl IntoResponse {
    let jar = axum_extra::extract::cookie::CookieJar::from_headers(req.headers());

    if let Some(cookie) = jar.get("admin_session") {
        if cookie.value() == "true" {
            return next.run(req).await;
        }
    }

    Redirect::to("/login?error=unauthorized").into_response()
}

async fn login_submit(
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    if form.password == "eunsoo" {
        let cookie = Cookie::build(("admin_session", "true"))
            .path("/")
            .max_age(Duration::seconds(30))
            // .expires(SystemTime::now() + Duration::from_secs(30)) // 1 hour
            .http_only(true);
        (jar.add(cookie), Redirect::to("/admin")) // Return cookie + redirect as a tuple
    } else {
        
        (jar, Redirect::to("/login?error=invalid")) // Wrong password → redirect with error
    }
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    // Remove the cookie
    let jar = jar.remove(Cookie::from("admin_session"));
    (jar, Redirect::to("/login"))
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[derive(Debug, Deserialize)]
struct ItemAddForm {
    name: String,
    price: f64,
    image_path: String,
    quantity: i32,
}


#[derive(Debug, Deserialize)]
struct ItemEditForm {
    id: i32,
    name: String,
    price: f64,
    image_path: String,
    quantity: i32,
}

#[derive(Debug, Deserialize)]
struct ItemDeleteForm {
    id: i32,
}

#[derive(Debug, FromRow)]
struct Item {
    id: i32,
    name: String,
    price: f64,
    image_path: String,
    quantity: i32,
}

// Templates
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    title: &'static str,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    title: &'static str,
    subtitle: &'static str,
    error: Option<&'static str>,
}
async fn login(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let error = params.get("error").map(|s| s.as_str());

    let message = match error {
        Some("unauthorized") => Some("Please log in to access admin"),
        Some("invalid") => Some("Incorrect password"),
        _ => None,
    };

    let template = LoginTemplate {
        title: "MD Inventory Manager",
        subtitle: "Admin Login",
        error: message,
    };

    axum::response::Html(template.render().unwrap())
}


#[derive(Deserialize)]
struct LoginForm {
    password: String,
}

// async fn login_submit(Form(form): Form<LoginForm>) -> impl IntoResponse {
//     if form.password == "secret123" {
//         Redirect::to("/admin")
//     } else {
//         Redirect::to("/login")
//     }
// }

#[derive(Template)]
#[template(path = "inventory.html")]
struct InventoryTemplate { title: &'static str, subtitle: &'static str, items: Vec<Item> }

#[derive(Template)]
#[template(path = "inventory-edit.html")]
struct InventoryEditTemplate { title: &'static str, subtitle: &'static str, items: Vec<Item> }


#[derive(Template)]
#[template(path = "sales.html")]
struct SalesTemplate { title: &'static str, subtitle: &'static str }

#[derive(Template)]
#[template(path = "sale-tool.html")]
struct SaleToolTemplate { title: &'static str, subtitle: &'static str }

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate { title: &'static str, subtitle: &'static str }

// Handlers
async fn home() -> impl IntoResponse {
    let template = HomeTemplate { title: "MD Inventory Manager" };
    axum::response::Html(template.render().unwrap())
}

async fn inventory(State(state): State<AppState>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, name, quantity, price, image_path FROM inventory"
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    println!("Fetched items: {:?}", items);
    let template = InventoryTemplate {
        title: "MD Inventory Manager",
        subtitle: "Inventory Overview",
        items: items,
    };
    axum::response::Html(template.render().unwrap())
}

async fn inventory_edit(State(state): State<AppState>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, name, quantity, price, image_path FROM inventory"
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    print!("Fetched items: {:?}", items);
    let template = InventoryEditTemplate {
        title: "MD Inventory Manager",
        subtitle: "Edit Inventory",
        items: items,
    };
    axum::response::Html(template.render().unwrap())
}


async fn inventory_add_submit(
    State(state): State<AppState>,
    Form(form): Form<ItemAddForm>,
) -> impl IntoResponse {
    println!("Received add form data: {:?}", form);
    let price: f64 = form.price as f64;
    println!("Price: {}", price);

    sqlx::query(
        "INSERT INTO inventory (name, image_path, price, quantity) VALUES (?, ?, ?, ?)"
    )
    .bind(&form.name)
    .bind(&form.image_path)
    .bind(price)
    .bind(form.quantity)
    .execute(&state.db)
    .await
    .unwrap();

    Redirect::to("/admin/inventory-edit")
}

async fn inventory_edit_submit(
    State(state): State<AppState>,
    Form(form): Form<ItemEditForm>,
) -> impl IntoResponse {
    println!("Received edit form data: {:?}", form);
    let price: f64 = form.price as f64;
    println!("Price: {}", price);

    sqlx::query("UPDATE inventory SET name = ?, price = ?, quantity = ?, image_path = ? WHERE id = ?")
        .bind(&form.name)
        .bind(price)
        .bind(form.quantity)
        .bind(&form.image_path)
        .bind(&form.id)
        .execute(&state.db)
        .await
        .unwrap();

    Redirect::to("/admin/inventory-edit")
}

async fn inventory_delete_submit(
    State(state): State<AppState>,
    Form(form): Form<ItemDeleteForm>,
) -> impl IntoResponse {
    println!("Received delete form data: {:?}", form);
    sqlx::query("DELETE FROM inventory WHERE id = ?")
        .bind(&form.id)
        .execute(&state.db)
        .await
        .unwrap();

    Redirect::to("/admin/inventory-edit")
}

async fn sales() -> impl IntoResponse {
    let template = SalesTemplate {
        title: "MD Inventory Manager",
        subtitle: "Sale History",
    };
    axum::response::Html(template.render().unwrap())
}

async fn sale_tool() -> impl IntoResponse {
    let template = SaleToolTemplate {
        title: "MD Inventory Manager",
        subtitle: "Sale Tool",
    };
    axum::response::Html(template.render().unwrap())
}

async fn admin_home() -> impl IntoResponse {
    let template = AdminTemplate {
        title: "MD Inventory Manager",
        subtitle: "Admin",
    };
    axum::response::Html(template.render().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite:inventory.db").await?;
    let state = AppState { db: pool };
    // let static_service = get_service(ServeDir::new("static")).handle_error(|_error| async {
    //     (StatusCode::INTERNAL_SERVER_ERROR, "Static file error")
    // });
    // let resource_service = get_service(ServeDir::new("resources")).handle_error(|_error| async {
    //     (StatusCode::INTERNAL_SERVER_ERROR, "Resource file error")
    // });

    // Admin routes (all protected)
    let admin_routes = Router::new()
        .route("/inventory-edit", get(inventory_edit))
        .route("/", get(admin_home))
        .route("/inventory/add", post(inventory_add_submit))
        .route("/inventory/edit", post(inventory_edit_submit))
        .route("/inventory/delete", post(inventory_delete_submit))
        .layer(axum::middleware::from_fn(admin_auth));


    let app = Router::new()
        .route("/", get(home))
        .route("/inventory", get(inventory))
        .route("/sales", get(sales))
        .route("/sale-tool", get(sale_tool))
        .route("/login", get(login).post(login_submit))
        .route("/logout", post(logout))
        .nest("/admin", admin_routes) // mount protected admin routes
        // .route("/inventory-edit", get(inventory_edit))
        // .route("/admin/inventory/add", post(inventory_add_submit))
        // .route("/admin/inventory/edit", post(inventory_edit_submit))
        // .route("/admin/inventory/delete", post(inventory_delete_submit))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/resources", ServeDir::new("resources"))
        // .nest_service("/static", static_service)
        // .nest_service("/resources", resource_service)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);


    // Run the server
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app)
        .await?;

    Ok(())
}
