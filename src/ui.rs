use crate::response::{BasicErrorResponse, CodeGrantResponse};
use std::borrow::Cow;

pub(crate) struct Headings<'a> {
    pub title: &'a str,
    pub subheader: Cow<'a, str>,
}

impl Headings<'static> {
    pub const fn new(title: &'static str, subheader: &'static str) -> Headings<'static> {
        Headings {
            title,
            subheader: Cow::Borrowed(subheader),
        }
    }
}

pub(crate) trait ToHeadings {
    fn to_headings(&self) -> Headings<'_>;
}

impl ToHeadings for CodeGrantResponse {
    fn to_headings(&self) -> Headings {
        Headings {
            title: "You are now logged in.",
            subheader: Cow::Borrowed("Please close the window."),
        }
    }
}

impl ToHeadings for BasicErrorResponse {
    fn to_headings(&self) -> Headings {
        let error = self.error().as_ref();
        let subheader = match (self.error_description(), self.error_uri()) {
            (None, None) => Cow::Borrowed(error),
            (Some(description), None) => Cow::Owned(format!("{error}: {description}")),
            (None, Some(uri)) => Cow::Owned(format!("{error} ({uri})")),
            (Some(description), Some(uri)) => Cow::Owned(format!("{error}: {description} ({uri})")),
        };
        Headings {
            title: "Login failed.",
            subheader,
        }
    }
}
