#[derive(Debug, PartialEq)]
pub struct Markdown<'a> {
    components: Vec<Component<'a>>,
}

impl<'a> Markdown<'a> {
    pub fn parse(input: &'a str) -> Markdown {
        Markdown {
            components: vec![Component::Title(&input[2..])],
        }
    }

    pub fn components(&'a self) -> impl Iterator<Item = &Component<'a>> {
        self.components.iter()
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

        assert_eq!(result, &Component::Title("Hello World"));

        let title = "# Good bye";
        let sut = Markdown::parse(title);

        let result = sut.components().next().unwrap();

        assert_eq!(result, &Component::Title("Good bye"));
    }
}
