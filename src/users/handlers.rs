use diesel::prelude::*;
use diesel::{delete, update};
use rocket::http::Status;
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::json::Json;

use crate::auth::crypto::*;
use crate::guards::*;
use crate::ObservDbConn;

use super::models::*;
use super::templates::*;

#[get("/users/<h>")]
pub fn user(conn: ObservDbConn, l: MaybeLoggedIn, h: i32) -> Option<UserTemplate> {
    use crate::schema::users::dsl::*;

    Some(UserTemplate {
        logged_in: l.user(),
        user: users
            .find(h)
            .first(&*conn)
            .optional()
            .expect("Failed to get user from database")?,
    })
}

#[get("/users/<h>", rank = 2)]
pub fn user_by_handle(conn: ObservDbConn, _l: MaybeLoggedIn, h: String) -> Option<Redirect> {
    use crate::schema::users::dsl::*;

    let u: User = users
        .filter(handle.like(h))
        .first(&*conn)
        .optional()
        .expect("Failed to get user from database")?;

    Some(Redirect::to(format!("/users/{}", u.id)))
}

#[get("/users/<h>/edit")]
pub fn edituser(conn: ObservDbConn, l: UserGuard, h: i32) -> Option<EditUserTemplate> {
    use crate::schema::users::dsl::*;

    Some(EditUserTemplate {
        logged_in: Some(l.0),
        user: users
            .find(h)
            .first(&*conn)
            .optional()
            .expect("Failed to get user from database")?,
    })
}

#[put("/users/<h>", data = "<edituser>")]
pub fn edituser_put(
    conn: ObservDbConn,
    l: UserGuard,
    h: i32,
    edituser: Form<NewUser>,
) -> Result<Redirect, Status> {
    let l = l.0;
    let mut edituser = edituser.into_inner();

    use crate::schema::users::dsl::*;
    // Get some more info about the edited user
    let (esalt, phash, etier) = users
        .find(h)
        .select((salt, password_hash, tier))
        .first(&*conn)
        .expect("Failed to get user from database");

    if l.tier > 1 || l.id == h {
        if edituser.password_hash.is_empty() {
            edituser.salt = esalt;
            edituser.password_hash = phash;
        } else {
            edituser.salt = gen_salt();
            edituser.password_hash = hash_password(edituser.password_hash, &edituser.salt);
        }

        // if the logged in user can't change tiers
        // of if it's the admin user
        // don't change tiers
        if !(l.tier > 1) || h == 0 {
            edituser.tier = etier;
        }

        update(users.find(h))
            .set(&edituser)
            .execute(&*conn)
            .expect("Failed to update user in database");

        Ok(Redirect::to(format!("/users/{}", edituser.handle)))
    } else {
        Err(Status::Unauthorized)
    }
}

#[delete("/users/<h>")]
pub fn user_delete(conn: ObservDbConn, _l: AdminGuard, h: i32) -> Redirect {
    use crate::schema::users::dsl::*;
    delete(users.find(h))
        .execute(&*conn)
        .expect("Failed to delete user from database");
    Redirect::to("/users")
}

#[get("/users?<s>")]
pub fn users(conn: ObservDbConn, l: MaybeLoggedIn, s: Option<String>) -> UsersListTemplate {
    UsersListTemplate {
        logged_in: l.user(),
        users: filter_users(&*conn, s),
    }
}

#[get("/users.json?<s>")]
pub fn users_json(conn: ObservDbConn, s: Option<String>) -> Json<Vec<User>> {
    Json(filter_users(&*conn, s))
}

pub fn filter_users(conn: &SqliteConnection, term: Option<String>) -> Vec<User> {
    use crate::schema::users::dsl::*;

    if let Some(term) = term {
        let sterm = format!("%{}%", term);
        let filter = real_name
            .like(&sterm)
            .or(email.like(&sterm))
            .or(handle.like(&sterm));
        users.filter(filter).load(conn)
    } else {
        users.load(conn)
    }
    .expect("Failed to get users")
}
