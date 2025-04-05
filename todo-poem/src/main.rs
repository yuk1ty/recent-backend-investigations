use std::net::Ipv4Addr;

use anyhow::Context;
use poem::{
    get, handler, http::StatusCode, listener::TcpListener, web::Data, EndpointExt, Result, Route,
    Server,
};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    Object, OpenApi, OpenApiService,
};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

struct Api;

#[derive(Object, sqlx::FromRow)]
struct Todo {
    id: Uuid,
    title: String,
    description: String,
    done: bool,
}

#[derive(Object)]
struct CreateTodoReq {
    title: String,
    description: String,
    done: bool,
}

#[OpenApi]
impl Api {
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<&'static str> {
        PlainText("Hello World")
    }

    #[oai(path = "/todo", method = "get")]
    async fn list(&self, pool: Data<&Pool<Postgres>>) -> Result<Json<Vec<Todo>>> {
        let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todo")
            .fetch_all(pool.0)
            .await
            .with_context(|| "Failed to fetch todos")?;
        Ok(Json(todos))
    }

    #[oai(path = "/todo/:id", method = "get")]
    async fn find_todo_by_id(
        &self,
        pool: Data<&Pool<Postgres>>,
        id: Path<i32>,
    ) -> Result<Json<Option<Todo>>> {
        let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todo WHERE id = $1")
            .bind(id.0)
            .fetch_optional(pool.0)
            .await
            .with_context(|| "Failed to fetch todo")?;
        Ok(Json(todo))
    }

    #[oai(path = "/todo", method = "post")]
    async fn create_todo(
        &self,
        pool: Data<&Pool<Postgres>>,
        todo: Json<CreateTodoReq>,
    ) -> Result<()> {
        sqlx::query_as::<_, Todo>(
            "INSERT INTO todo (id, title, description, done) VALUES ($1, $2, $3)",
        )
        .bind(Uuid::new_v4())
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.done)
        .fetch_one(pool.0)
        .await
        .with_context(|| "Failed to create todo")?;
        Ok(())
    }

    #[oai(path = "/todo/:id", method = "delete")]
    async fn delete_todo(&self, pool: Data<&Pool<Postgres>>, id: Path<Uuid>) -> Result<()> {
        sqlx::query("DELETE FROM todo WHERE id = $1")
            .bind(id.0)
            .execute(pool.0)
            .await
            .with_context(|| "Failed to delete todo")?;
        Ok(())
    }
}

// あんまりわかってないのだけど、OpenAPIの場合StatusCodeを直で返せない。
// ApiResponseトレイトを実装してくれ、というコンパイルエラーが出る。
// 多分実装忘れなんじゃないか？という気がしている。
#[handler]
async fn health_check() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPool::connect("postgres://postgres:password@localhost:5432/todo_app")
        .await
        .with_context(|| "Failed to connect to database")?;

    let addr = format!("{}:8081", Ipv4Addr::LOCALHOST);

    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server(format!("http://{}", addr));
    let ui = api_service.redoc();
    let app = Route::new()
        .at("/hc", get(health_check))
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool);

    Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .with_context(|| "Failed to run server")
}
