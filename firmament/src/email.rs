use aws_sdk_ses::types::{Body, Content, Destination, Message};

use crate::aws::ses;

pub type SendEmailError = ses::error::SdkError<ses::operation::send_email::SendEmailError>;

#[derive(Clone, Debug, Default)]
pub struct Email {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: String,
}

impl Email {
    pub const fn new() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            text: String::new(),
            html: String::new(),
        }
    }

    #[inline]
    pub fn set_from(&mut self, value: &str) {
        self.from.replace_range(.., value);
    }

    #[inline]
    pub fn set_to(&mut self, value: &str) {
        self.to.replace_range(.., value);
    }

    #[inline]
    pub fn set_subject(&mut self, value: &str) {
        self.subject.replace_range(.., value);
    }

    #[inline]
    pub fn set_text(&mut self, value: &str) {
        self.text.replace_range(.., value);
    }

    #[inline]
    pub fn set_html(&mut self, value: &str) {
        self.html.replace_range(.., value);
    }

    pub async fn send(self) -> Result<(), SendEmailError> {
        let destination = Destination::builder().to_addresses(self.to).build();
        let mut body = Body::builder();

        fn utf8_content(value: String) -> Content {
            Content::builder()
                .charset("UTF-8")
                .data(value)
                .build()
                .unwrap()
        }

        if !self.text.is_empty() {
            body = body.text(utf8_content(self.text));
        }

        if !self.html.is_empty() {
            body = body.html(utf8_content(self.html));
        }

        let message = Message::builder()
            .subject(utf8_content(self.subject))
            .body(body.build())
            .build();

        let result = ses::client()
            .await
            .send_email()
            .source(self.from)
            .destination(destination)
            .message(message)
            .send()
            .await;

        result.map(|_| ())
    }
}
