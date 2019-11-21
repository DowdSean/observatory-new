//! HTTP handlers for authentication

use diesel::insert_into;
use diesel::prelude::*;
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::Redirect;

use crate::guards::*;
use crate::models::NewRelationGroupUser;
use crate::models::{NewUser, User};
use crate::templates::{is_reserved, FormError};
use crate::ObservDbConn;

use super::crypto::*;
use super::templates::*;

/// GET handler for `/signup`
#[get("/signup?<e>")]
pub fn signup(l: MaybeLoggedIn, e: Option<FormError>) -> SignUpTemplate {
    SignUpTemplate {
        logged_in: l.user(),
        error: e,
    }
}

/// User's signup info
///
/// Struct used to parse information at signup.
#[derive(Debug, FromForm)]
pub struct SignUpForm {
    email: String,
    password: String,
    password_repeat: String,
    real_name: String,
    handle: String,
    mmost: String,
}

impl From<SignUpForm> for NewUser {
    fn from(f: SignUpForm) -> Self {
        let mut newuser = Self::default();

        newuser.email = f.email;
        newuser.real_name = f.real_name;
        newuser.handle = f.handle;
        newuser.mmost = f.mmost;

        let (pass, salt) = hash_password(f.password);
        newuser.salt = salt;
        newuser.password_hash = pass;

        newuser.tier = 0;
        newuser.active = true;

        newuser.former = false;
        newuser.extrn = false;

        newuser
    }
}

/// POST handler for `/signup`
///
/// Creates a new user in the database with the information provided by the
/// POSTed form and logs them in.
///
/// If all goes well then it redirects to `/` otherwise back to the same page.
#[post("/signup", data = "<form>")]
pub fn signup_post(conn: ObservDbConn, mut cookies: Cookies, form: Form<SignUpForm>) -> Redirect {
    let mut form = form.into_inner();
    form.handle.truncate(39);
    form.mmost.truncate(22);
    // Make sure the password is properly repeated
    if form.password != form.password_repeat {
        return Redirect::to(format!("/signup?e={}", FormError::PasswordMismatch));
    }

    let newuser = NewUser::from(form);

    use crate::schema::users::dsl::*;

    if let Err(e) = is_reserved(&*newuser.handle) {
        return Redirect::to(format!("/signup?e={}", e));
    }

    // Check if user's email is already signed up
    if users
        .filter(&email.eq(&*newuser.email))
        .first::<User>(&*conn)
        .optional()
        .expect("Failed to get user from database")
        .is_some()
    {
        return Redirect::to(format!("/signup?e={}", FormError::EmailExists));
    }

    // Check if user's github is already signed up
    if users
        .filter(&handle.eq(&*newuser.handle))
        .first::<User>(&*conn)
        .optional()
        .expect("Failed to get user from database")
        .is_some()
    {
        return Redirect::to(format!("/signup?e={}", FormError::GitExists));
    }

    // Check if user's mattermost is already signed up
    if users
        .filter(&mmost.eq(&*newuser.mmost))
        .first::<User>(&*conn)
        .optional()
        .expect("Failed to get user from database")
        .is_some()
    {
        return Redirect::to(format!("/signup?e={}", FormError::MmostExists));
    }

    // Insert the new user into the database
    insert_into(users)
        .values(&newuser)
        .execute(&*conn)
        .expect("Failed to add user to database");

    let user: User = users
        .filter(&email.eq(&*newuser.email))
        .first(&*conn)
        .expect("Failed to get user from database");
    {
        use crate::schema::relation_group_user::dsl::*;
        insert_into(relation_group_user)
            .values(&NewRelationGroupUser {
                group_id: 0,
                user_id: user.id,
            })
            .execute(&*conn)
            .expect("Failed to insert new relation into database");
    }

    cookies.add_private(Cookie::new("user_id", format!("{}", user.id)));

    audit_logger!(
        "User {} [{}] has registered for an account",
        user.id,
        user.email
    );

    Redirect::to(format!("/users/{}", user.id))
}

/// GET handler for `/login`
#[get("/login?<e>")]
pub fn login(l: MaybeLoggedIn, e: Option<FormError>) -> LogInTemplate {
    LogInTemplate {
        logged_in: l.user(),
        error: e,
    }
}

/// User's creditentials
///
/// Used to parse the incoming form for `login_post`
#[derive(Default, FromForm)]
pub struct LogInForm {
    pub email: String,
    pub password: String,
}

/// POST handler for `/login`
///
/// This handler attempts to verify the creditionals POSTed to it and then
/// on succes redirects to `/` otherwise back to the same page.
#[post("/login?<to>", data = "<creds>")]
pub fn login_post(
    conn: ObservDbConn,
    mut cookies: Cookies,
    creds: Form<LogInForm>,
    to: Option<String>,
) -> Redirect {
    use crate::schema::users::dsl::*;

    let creds = creds.into_inner();

    let to = to.unwrap_or_else(|| String::from("/"));

    // If we find the user
    if let Some(user) = users
        .filter(&email.eq(creds.email))
        .first::<User>(&*conn)
        .optional()
        .expect("Failed to get user from database")
    {
        // Verify the password
        if verify_password(creds.password, user.password_hash, user.salt) {
            cookies.add_private(Cookie::new("user_id", format!("{}", user.id)));
            Redirect::to(to)
        } else {
            Redirect::to(format!("/login?to={}&e={}", to, FormError::Password))
        }
    } else {
        Redirect::to(format!("/login?to={}&e={}", to, FormError::Email))
    }
}

#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/")
}
