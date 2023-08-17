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

impl<'a> Headings<'a> {
    pub fn html(&self) -> String {
        format!(
            r#"<html>
        <body>
            <div style="
                width: 100%;
                top: 50%;
                margin-top: 100px;
                text-align: center;
                font-family: sans-serif;
            ">
                <h1>{}</h1>
                <h2>{}</h2>
            </div>
        </body>
    </html>"#,
            self.title, self.subheader
        )
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
