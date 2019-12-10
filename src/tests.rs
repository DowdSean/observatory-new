use super::*;
use crate::auth::crypto::*;
use crate::models::*;
use diesel::insert_into;
use diesel::delete;
use diesel::prelude::*;
use rocket::config::{Config, Environment, LoggingLevel, Value};
use rocket::http::Status;
use rocket::local::Client;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Embed;

// Get all handlers in case we need to test other functions
pub use crate::attend::handlers::*;
pub use crate::auth::handlers::*;
pub use crate::calendar::handlers::*;
pub use crate::groups::handlers::*;
pub use crate::news::handlers::*;
pub use crate::projects::handlers::*;
pub use crate::users::handlers::*;

// Embed the Migrations into the binary
embed_migrations!("migrations/sqlite");

// Static connection variable

fn create_connection_url(client: &rocket::local::Client) -> String {
    String::from(
        client
            .rocket()
            .config()
            .get_table("databases")
            .unwrap()
            .get("sqlite_observ")
            .unwrap()
            .get("url")
            .unwrap()
            .as_str()
            .unwrap(),
    )
}

fn setup(test_name: String) -> Option<rocket::Config> {
    let mut db_path = String::from("./");
    db_path.push_str(test_name.as_str());
    db_path.push_str("/");

    let db_path_exists = Path::new(db_path.as_str()).is_dir();

    if !db_path_exists {
        fs::create_dir(db_path.as_str()).expect("Dir Creation Error");
    }

    let mut db_file_path = String::from(db_path.as_str());
    db_file_path.push_str(test_name.as_str());
    db_file_path.push_str(".sqlite");

    let db_file_exists = Path::new(db_file_path.as_str()).is_file();

    if !db_file_exists {
        fs::File::create(db_file_path.as_str()).expect("File Creation Error");
    }

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();
    database_config.insert("url", Value::from(db_file_path.as_str()));
    databases.insert("sqlite_observ", Value::from(database_config));

    let config = Config::build(Environment::Development)
        .root("/")
        .address("localhost")
        .port(8000)
        .log_level(LoggingLevel::Normal)
        .extra("databases", databases)
        .finalize()
        .unwrap();

    Some(config)
}

fn cleanup(test_name: String) {
    let mut db_path_string = String::from("./");
    db_path_string.push_str(test_name.as_str());
    db_path_string.push_str("/");

    let mut db_file_string = String::from(db_path_string.as_str());
    db_file_string.push_str(test_name.as_str());
    db_file_string.push_str(".sqlite");

    fs::remove_file(db_file_string).expect("File Deletion Error");

    fs::remove_dir(db_path_string).expect("Dir Deletion Error");
}

#[test]
fn launch() {
    let config = setup(String::from("test_launch"));

    let _client = Client::new(rocket(config)).unwrap();
    let conn_url = create_connection_url(&_client);

    let conn = SqliteConnection::establish(conn_url.as_str())
        .expect("Failed to connect to database in LaunchTest");
    embedded_migrations::run(&conn).expect("Failed to run embedded migrations");

    let response = _client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);

    cleanup(String::from("test_launch"));
}

#[test]
fn check_static_content() {
    let config = setup(String::from("test_static_content"));

    let _client = Client::new(rocket(config)).unwrap();
    let conn_url = create_connection_url(&_client);

    let conn = SqliteConnection::establish(conn_url.as_str())
        .expect("Failed to connect to database in StaticContentTest");
    embedded_migrations::run(&conn).expect("Failed to run embedded migrations");

    let mut response = _client.get("/").dispatch();
    assert!(response.body().is_some());

    let body_str = response.body_string().unwrap();
    assert!(body_str.contains("chat.rcos.io"));

    Embed::get("img/favicon.webp").unwrap();

    cleanup(String::from("test_static_content"));
}

// Tests the addition, editing, and deletion of a user
#[test]
fn add_user() {

    // cleanup(String::from("test_add_user"));

    let config = setup(String::from("test_add_user"));

    let _client = Client::new(rocket(config)).unwrap();
    let conn_url = create_connection_url(&_client);

    let conn = SqliteConnection::establish(conn_url.as_str())
        .expect("Failed to connect to database in AddUserTest");
    embedded_migrations::run(&conn).expect("Failed to run embedded migrations");

    use crate::schema::users::dsl::*;
    let pass = String::from("thisisapassword");

    let (phash, psalt) = hash_password(pass);

    let nu = NewUser {
        real_name: String::from("John Doe"),
        handle: String::from("JD1"),
        password_hash: phash,
        salt: psalt,
        bio: String::from("This is a test user. Do not disturb."),
        email: String::from("doej@test-rcos.io"),
        tier: 0,
        active: true,
        mmost: String::from("JDMM"),
        former: false,
        extrn: false,
    };

    insert_into(users)
        .values(&nu)
        .execute(&conn)
        .expect("Failed to add user to database");

    let user: User = users
        .filter(&email.eq(&*nu.email))
        .first(&conn)
        .expect("Failed to get user from database");
    {
        use crate::schema::relation_group_user::dsl::*;
        insert_into(relation_group_user)
            .values(&NewRelationGroupUser {
                group_id: 0,
                user_id: user.id,
            })
            .execute(&conn)
            .expect("Failed to insert new relation into database");
    }

    assert_eq!("JD1".to_string(), user.handle);
   // User update: create new instance with updated values and enter into db
   // or edit values directly somehow?
     
 
    // User deletion
    delete(users.filter(&email.eq(&*nu.email)))
        .execute(&conn)
        .expect("Failed to delete user from database");
   
    cleanup(String::from("test_add_user"));
}



  
// Tests the addition, editing, and deletion of a group 
#[test]
fn add_group() {
    let config = setup(String::from("test_add_group"));

    let _client = Client::new(rocket(config)).unwrap();
    let conn_url = create_connection_url(&_client);

    let conn = SqliteConnection::establish(conn_url.as_str())
        .expect("Failed to connect to database in AddGroupTest");
    embedded_migrations::run(&conn).expect("Failed to run embedded migrations");
    use crate::schema::groups::dsl::*;
    let nu = NewGroup {
        name: String::from("Test Group"),
        owner_id: 0,
        location: Some(String::from("DCC 318")),
    };
    insert_into(groups)
        .values(&nu)
        .execute(&conn)
        .expect("Failed to add group to database");

    let group: Group = groups
        //.filter(&email.eq(&*nu.email))
        .first(&conn)
        .expect("Failed to get group from database");
    {
        use crate::schema::relation_group_user::dsl::*;
        insert_into(relation_group_user)
            .values(&NewRelationGroupUser {
                group_id: group.id,
                user_id: 1,
            })
            .execute(&conn)
            .expect("Failed to insert new relation into database");
    }
    // Update to group name and meeting location  
    
    // Group deletion
    delete(groups.filter(id.eq(1)))
        .execute(&conn)
        .expect("Failed to delete group from database");
    cleanup(String::from("test_add_group"));
} 
