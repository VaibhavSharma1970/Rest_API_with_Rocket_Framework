// #![feature(proc_macro_hygiene, decl_macro)]

// #[macro_use] extern crate rocket;

// // use rocket::{get, routes};
// // use rocket::http::Status;
// // use rocket::response::status;

// #[get("/")]
// fn index() -> &'static str {
//     "Hello, Rocket!"
// }

// // #[get("/user/<id>")]
// // fn get_user(id: i32) -> String {
// //     format!("User ID: {}", id)
// // }

// // #[catch(404)]
// // fn not_found() -> status::Custom<&'static str> {
// //     status::Custom(Status::NotFound, "Not Found")
// // }

// fn main() {
//     {
//     let _db_connection = rusqlite::Connection::open("data.sqlite");
    
//     _db_connection.expect("REASON").execute("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, item varchar(64) not null );",rusqlite:: ).unwrap();
// }
//     rocket::ignite().mount("/", routes![index]).launch();
//         // .register(catchers![not_found])
        
// }

#![feature(proc_macro_hygiene, decl_macro)]

use rocket::*;
use rocket_contrib::json::Json;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
struct ToDoList {
    items: Vec<ToDoItem>,
}

#[derive(Serialize)]
struct ToDoItem {
    id: i64,
    item: String,
}

#[derive(Serialize)]
struct StatusMessage {
    message: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/todo")]
fn fetch_all_todo_items() -> Result<Json<ToDoList>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"));
        }
    };

    let mut statement = match db_connection.prepare("select id, item from todo_list;") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare query".into()),
    };

    let results = statement.query_map(rusqlite::named_params! {}, |row| {
        Ok(ToDoItem {
            id: row.get(0)?,
            item: row.get(1)?,
        })
    });

    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(ToDoList { items })),
                Err(_) => Err("Could not collect items".into()),
            }
        }
        Err(_) => Err("Failed to fetch todo items".into()),
    }
}

#[post("/todo", format = "json", data = "<item>")]
fn add_todo_item(item: Json<String>) -> Result<Json<StatusMessage>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"));
        }
    };

    let mut statement =
        match db_connection.prepare("insert into todo_list (id, item) values (null, $1);") {
            Ok(statement) => statement,
            Err(_) => return Err("Failed to prepare query".into()),
        };
    let results = statement.execute(&[&item.0]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {
            message: format!("{} rows inserted!", rows_affected),
        })),
        Err(_) => Err("Failed to insert todo item".into()),
    }
}

#[delete("/todo/<id>")]
fn remove_todo_item(id: i64) -> Result<Json<StatusMessage>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"));
        }
    };

    let mut statement = match db_connection.prepare("delete from todo_list where id = $1;") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute(&[&id]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {
            message: format!("{} rows deleted!", rows_affected),
        })),
        Err(_) => Err("Failed to delete todo item".into()),
    }
}

fn main() {
    {
        let db_connection = Connection::open("data.sqlite").unwrap();

        db_connection
            .execute(
                "create table if not exists todo_list (
                    id integer primary key,
                    item varchar(64) not null
                );",
                rusqlite::named_params!(),
            )
            .unwrap();
    }

    rocket::ignite()
        .mount(
            "/",
            routes![index, fetch_all_todo_items, add_todo_item, remove_todo_item],
        )
        .launch();
}

