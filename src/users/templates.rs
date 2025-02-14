//!

use super::models::*;
use crate::models::{Group, Project};

#[allow(unused_imports)]
use crate::models::Attendable;
#[allow(unused_imports)]
use crate::templates::{filters, OptUser};

#[derive(Template)]
#[template(path = "user/user.html")]
pub struct UserTemplate {
    pub logged_in: OptUser,
    pub user: User,
    pub projects: Vec<Project>,
    pub summary: GradeSummary,
    pub groups: Vec<Group>,
}

#[derive(Template)]
#[template(path = "user/edit-user.html")]
pub struct EditUserTemplate {
    pub logged_in: OptUser,
    pub user: User,
}

#[derive(Template)]
#[template(path = "user/users-list.html")]
pub struct UsersListTemplate {
    pub logged_in: OptUser,
    pub users: Vec<User>,
}
