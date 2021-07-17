// Render object as String
pub trait Render {
    fn render(&self) -> String;
}

impl Render for String {
    fn render(&self) -> String {
        self.to_owned()
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[derive(Render)]
    #[render(template = "key = {}, value plus one = {}")]
    struct Record {
        #[render(pos = 0)]
        key: String,
        #[render(pos = 1, expr = "value + 1")]
        value: i32,
    }

    #[test]
    fn test_render_record() {
        let r = Record {
            key: "foo".to_owned(),
            value: 1,
        };
        assert_eq!("key = foo, value plus one = 2", &r.render())
    }
}
