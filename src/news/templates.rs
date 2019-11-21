use super::models::*;
#[allow(unused_imports)]
use crate::templates::filters;
use crate::templates::{FormError, OptUser};

#[derive(Template)]
#[template(path = "news/news.html")]
pub struct NewsTemplate {
    pub logged_in: OptUser,
    pub stories: Vec<NewsStory>,
}

#[derive(Template)]
#[template(path = "news/newsstory.html")]
pub struct NewsStoryTemplate {
    pub logged_in: OptUser,
    pub story: NewsStory,
}

#[derive(Template)]
#[template(path = "news/new-newsstory.html")]
pub struct NewNewsStoryTemplate {
    pub logged_in: OptUser,
    pub error: Option<FormError>,
}

#[derive(Template)]
#[template(path = "news/edit-newsstory.html")]
pub struct EditNewsStoryTemplate {
    pub logged_in: OptUser,
    pub story: NewsStory,
    pub error: Option<FormError>,
}

use crate::models::Event;
#[derive(Template)]
#[template(path = "news/slides.html")]
pub struct SlidesTemplate {
    pub events: Vec<Event>,
    pub news: Vec<NewsStory>,
}
