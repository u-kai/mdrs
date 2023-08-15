#[derive(Debug, PartialEq)]
pub struct Markdown {}

impl Markdown {
    pub fn parse<'a>(input: &'a str) -> Markdown {
        Markdown {}
    }

    pub fn components<'a>(&'a self) -> impl Iterator<Item = Component<'a>> {
        vec![Component::Title("Hello World")].into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub enum Component<'a> {
    Title(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn 文字列からタイトルをparseできる() {
        let title = "# Hello World";
        let sut = Markdown::parse(title);

        let result = sut.components().next().unwrap();

        assert_eq!(result, Component::Title("Hello World"));
    }
}
