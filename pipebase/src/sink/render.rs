// render template with object field as parameter
pub trait Render {
    fn render(&self) -> String;
}
