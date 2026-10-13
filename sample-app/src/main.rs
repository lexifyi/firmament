use firmament::Email;

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut email = Email::new();

    email.set_from("Sample app <no-reply@lexi.fyi>");
    email.set_to("contact@lexi.fyi");
    email.set_subject("hey :)");
    email.set_html("<p>Hello world!</p>");

    email.send().await?;

    Ok(())
}
