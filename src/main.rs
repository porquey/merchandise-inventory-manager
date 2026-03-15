use axum::{
    extract::State,
    // response::Html,
    routing::get,
    routing::post,
    routing::get_service,
    Router,
    // Extension,
    Form,
    response::Redirect,
    http::StatusCode,
};
use std::net::SocketAddr;
use tokio;
use askama::Template;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;
use sqlx::FromRow;
use serde::Deserialize;

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
async fn home() -> impl axum::response::IntoResponse {
    let template = HomeTemplate { title: "MD Inventory Manager" };
    axum::response::Html(template.render().unwrap())
}

async fn inventory(State(state): State<AppState>) -> impl axum::response::IntoResponse {
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

async fn inventory_edit(State(state): State<AppState>) -> impl axum::response::IntoResponse {
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
) -> impl axum::response::IntoResponse {
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

    Redirect::to("/inventory-edit")
}

async fn inventory_edit_submit(
    State(state): State<AppState>,
    Form(form): Form<ItemEditForm>,
) -> impl axum::response::IntoResponse {
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

    Redirect::to("/inventory-edit")
}

async fn inventory_delete_submit(
    State(state): State<AppState>,
    Form(form): Form<ItemDeleteForm>,
) -> impl axum::response::IntoResponse {
    println!("Received delete form data: {:?}", form);
    sqlx::query("DELETE FROM inventory WHERE id = ?")
        .bind(&form.id)
        .execute(&state.db)
        .await
        .unwrap();

    Redirect::to("/inventory-edit")
}

async fn sales() -> impl axum::response::IntoResponse {
    let template = SalesTemplate {
        title: "MD Inventory Manager",
        subtitle: "Sale History",
    };
    axum::response::Html(template.render().unwrap())
}

async fn sale_tool() -> impl axum::response::IntoResponse {
    let template = SaleToolTemplate {
        title: "MD Inventory Manager",
        subtitle: "Sale Tool",
    };
    axum::response::Html(template.render().unwrap())
}

async fn admin() -> impl axum::response::IntoResponse {
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
    let static_service = get_service(ServeDir::new("static")).handle_error(|_error| async {
        (StatusCode::INTERNAL_SERVER_ERROR, "Static file error")
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/inventory", get(inventory))
        .route("/sales", get(sales))
        .route("/sale-tool", get(sale_tool))
        .route("/admin", get(admin))
        .route("/inventory-edit", get(inventory_edit))
        .route("/admin/inventory/add", post(inventory_add_submit))
        .route("/admin/inventory/edit", post(inventory_edit_submit))
        .route("/admin/inventory/delete", post(inventory_delete_submit))
        .nest_service("/static", static_service)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    // Run the server
    axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;

    Ok(())
}
