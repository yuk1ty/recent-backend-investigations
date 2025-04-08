mod migrations;

use cot::bytes::Bytes;
use cot::cli::CliMetadata;
use cot::db::migrations::SyncDynMigration;
use cot::db::{model, query, Auto, Model};
use cot::http::StatusCode;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandlerBuilder};
use cot::request::extractors::{Json, Path, RequestDb};
use cot::response::{Response, ResponseExt};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::{static_files, App, AppBuilder, Body, BoxedHandler, Project};
use rinja::Template;
use serde::{Deserialize, Serialize};

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> cot::Result<Response> {
    let index_template = IndexTemplate {};
    let rendered = index_template.render()?;

    Ok(Response::new_html(StatusCode::OK, Body::fixed(rendered)))
}

#[model]
pub struct Todo {
    // UUIDはまだ未対応らしい。
    #[model(primary_key)]
    id: Auto<i64>,
    title: String,
    description: String,
    done: bool,
}

#[derive(Deserialize)]
pub struct CreateTodoReq {
    title: String,
    description: String,
}

#[derive(Serialize)]
pub struct TodoRes {
    id: i64,
    title: String,
    description: String,
    done: bool,
}

async fn list_todos(RequestDb(db): RequestDb) -> cot::Result<Response> {
    let todos = Todo::objects().all(&db).await?;

    let response = Response::new_json(
        StatusCode::OK,
        &todos
            .iter()
            .map(|todo| TodoRes {
                id: todo.id.unwrap(),
                title: todo.title.clone(),
                description: todo.description.clone(),
                done: todo.done,
            })
            .collect::<Vec<TodoRes>>(),
    )?;
    Ok(response)
}

async fn create_todo(
    RequestDb(db): RequestDb,
    Json(req): Json<CreateTodoReq>,
) -> cot::Result<Response> {
    let mut todo = Todo {
        id: Auto::default(),
        title: req.title,
        description: req.description,
        done: false,
    };

    todo.insert(&db).await?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::empty())
        .unwrap())
}

async fn delete_todo(RequestDb(db): RequestDb, Path(id): Path<i64>) -> cot::Result<Response> {
    query!(Todo, $id == id).delete(&db).await?;
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

struct TodoCotApp;

impl App for TodoCotApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        cot::db::migrations::wrap_migrations(migrations::MIGRATIONS)
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            Route::with_handler_and_name("/todos", list_todos, "list-todo"),
            Route::with_handler_and_name("/todos", create_todo, "create-todo"),
            Route::with_handler_and_name("/todos/:id", delete_todo, "delete-todo"),
        ])
    }

    fn static_files(&self) -> Vec<(String, Bytes)> {
        static_files!("css/main.css")
    }
}

struct TodoCotProject;

impl Project for TodoCotProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn register_apps(&self, apps: &mut AppBuilder, _context: &RegisterAppsContext) {
        apps.register_with_views(TodoCotApp, "");
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &MiddlewareContext,
    ) -> BoxedHandler {
        handler
            .middleware(StaticFilesMiddleware::from_context(context))
            .middleware(AuthMiddleware::new())
            .middleware(SessionMiddleware::new())
            .middleware(LiveReloadMiddleware::from_context(context))
            .build()
    }
}

#[cot::main]
fn main() -> impl Project {
    TodoCotProject
}
