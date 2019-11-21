//! Misc HTML templates
//!
//! Various templates that do not belong to any of the other modules or
//! are "top-level" such as the index and static route.
//!
//! Template contain the state of the page they relate to and are returned
//! by a handler since they can be rendered to HTML.

use crate::models::User;
use crate::news::models::NewsStory;

/// Companion to `MaybeLoggedIn`
///
/// HTML File: `index.html`
///
/// This is a simple wrapper to act as the companion to `MaybeLoggedIn`
/// where that is a Guard and this is just the `User`.
pub type OptUser = Option<User>;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub logged_in: OptUser,
    pub announcement: Option<NewsStory>,
    pub version: &'static str,
}

/// Big text template
///
/// HTML File: `big.html`
///
/// This is a simple template that just shows the given text large
/// across the screen. Useful for attendance codes.
#[derive(Template)]
#[template(path = "big.html")]
pub struct BigTemplate {
    pub logged_in: OptUser,
    pub text: String,
}

use crate::models::GradeSummary;
use crate::models::Group;
use crate::models::Project;

/// User Dashboard template
///
/// HTML File: `dashboard.html`
///
/// This template shows the user's dashboard with the projects
/// and groups they are a part of as well as their grade summary.
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub logged_in: OptUser,
    pub projects: Vec<Project>,
    pub groups: Vec<Group>,
    pub summary: GradeSummary,
}

/// Site Map template
///
/// HTML File: `sitemap.html`

#[derive(Template)]
#[template(path = "sitemap.html")]
pub struct SitemapTemplate {}

//# Catcher Templates

/// 403 error template
///
/// HTML File: `catchers/403.html`
///
/// Just tells the user they are unauthorized.
#[derive(Template)]
#[template(path = "catchers/403.html")]
pub struct Error403Template {
    pub logged_in: OptUser,
}

/// 404 error template
///
/// HTML File: `catchers/404.html`
///
/// Just tells the user they are lost.
#[derive(Template)]
#[template(path = "catchers/404.html")]
pub struct Error404Template {
    pub logged_in: OptUser,
}

/// Filters namespace
///
/// Puts the filters in the proper namespace so that templates can use them.
/// Done like this so we can add custom filters later.
pub mod filters {
    pub use askama_filters::filters::*;
}

/// An error in an HTML form
///
/// This enum is used to represent errors in HTML forms.
/// Feedback is provided to the user using `form-error.html`.
/// Generally you pass this back as a GET argument to the form page.
#[derive(Debug)]
pub enum FormError {
    /// The email was incorrect or missing
    Email,
    /// The password was incorrect or missing
    Password,
    /// When you don't want to specify if it was an
    /// issue witht he email or the password
    Credentials,
    /// The password and it's repeat are not the same
    PasswordMismatch,
    /// The email is already in use by another user
    EmailExists,
    /// The github handle is already in use by another user
    GitExists,
    /// The mattermost handle is already in use by another user
    MmostExists,
    /// An attendance code does not exist or is used by a non-affiliated member
    InvalidCode,
    /// A date field was the wrong format invalid
    InvalidDate,
    /// You used a name that is reserved and can't be used
    ReservedName,
    /// Some other unknown error
    Other,
}

use std::fmt;

// Converts to a string
impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FormError::Email => "email",
                FormError::Password => "password",
                FormError::Credentials => "credentials",
                FormError::PasswordMismatch => "mismatch",
                FormError::EmailExists => "emailExists",
                FormError::GitExists => "gitExists",
                FormError::MmostExists => "mmostExists",
                FormError::InvalidCode => "code",
                FormError::InvalidDate => "date",
                FormError::ReservedName => "reserved",
                FormError::Other => "other",
            }
        )
    }
}

// Converts from a string
impl<T: AsRef<str>> From<T> for FormError {
    fn from(f: T) -> FormError {
        match f.as_ref() {
            "email" => FormError::Email,
            "password" => FormError::Password,
            "credentials" => FormError::Credentials,
            "mismatch" => FormError::PasswordMismatch,
            "emailExists" => FormError::EmailExists,
            "gitExists" => FormError::GitExists,
            "mmostExists" => FormError::MmostExists,
            "code" => FormError::InvalidCode,
            "date" => FormError::InvalidDate,
            "reserved" => FormError::ReservedName,
            "other" => FormError::Other,
            _ => FormError::Other,
        }
    }
}

use rocket::http::RawStr;
use rocket::request::FromFormValue;

// Let's a FormError be created from a Rocket Form.
impl<'v> FromFormValue<'v> for FormError {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<FormError, &'v RawStr> {
        Ok(FormError::from(form_value))
    }
}

pub const RESERVED_WORDS: &[&str] = &["new", "start", "rcos", "edit"];

pub fn is_reserved(word: &str) -> Result<&str, FormError> {
    if RESERVED_WORDS.contains(&&*word.to_lowercase()) || word.parse::<usize>().is_ok() {
        Err(FormError::ReservedName)
    } else {
        Ok(word)
    }
}
