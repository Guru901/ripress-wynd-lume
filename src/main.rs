use std::env;

use lume::{
    database::{Database, error::DatabaseError},
    define_schema,
};

use ripress::{app::App, types::RouterFns};
use wynd::wynd::{WithRipress, Wynd};

define_schema! {
    Users {
        id: u64,
        name: String,
        email: String,
        password: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    dotenv::dotenv().ok();
    let mut wynd: Wynd<WithRipress> = Wynd::new();
    let mut app = App::new();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url).await?;

    db.register_table::<Users>().await?;

    db.insert(Users {
        id: 1,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        password: "password".to_string(),
    })
    .execute()
    .await?;

    wynd.on_connection(|conn| async move {
        conn.on_text(|message, handle| async move {
            if message.data == "get_users" {
                let users = get_users().await;
                handle.send_text(users.join(", ")).await.unwrap();
            } else {
                handle
                    .send_text("Invalid message. Use 'get_users' to get users.")
                    .await
                    .unwrap();
            }
        });
    });

    app.get("/", |_, res| async move { res.ok().text("Hello World!") });
    app.get("/users", |_, res| async {
        let users = get_users().await;
        res.ok().json(&users)
    });

    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
    })
    .await;

    Ok(())
}

async fn get_users() -> Vec<String> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url).await.unwrap();

    let users = db
        .query::<Users, SelectUsers>()
        .select(SelectUsers::selected().name())
        .execute()
        .await
        .unwrap();

    let user_names = users
        .iter()
        .map(|user| user.get(Users::name()).unwrap())
        .collect::<Vec<String>>();

    user_names
}
