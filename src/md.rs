#[derive(Debug, PartialEq)]
pub struct Markdown<'a> {
    components: Vec<Component<'a>>,
}

impl<'a> Markdown<'a> {
    pub fn parse(input: &'a str) -> Markdown {
        let components = Markdown::parse_components(input);
        Markdown { components }
    }
    pub fn components(&'a self) -> impl Iterator<Item = &Component<'a>> {
        self.components.iter()
    }
    fn parse_components(input: &'a str) -> Vec<Component<'a>> {
        let mut components = Vec::new();
        for line in input.lines() {
            if line.starts_with("# ") {
                let title = line.trim_start_matches("# ");
                components.push(Component::Heading1(title));
            }
        }
        components
    }
}

#[derive(Debug, PartialEq)]
pub enum Component<'a> {
    Heading1(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn 複数の行をparseできる() {
        let title = r#"# Hello World
- foo
- bar
"#;
        let sut = Markdown::parse(title);

        let title = sut.components().next().unwrap();

        assert_eq!(title, &Component::Heading1("Hello World"));
    }
    #[test]
    fn 文字列からタイトルをparseできる() {
        let title = "# Hello World";
        let sut = Markdown::parse(title);

        let result = sut.components().next().unwrap();

        assert_eq!(result, &Component::Heading1("Hello World"));

        let title = "# Good bye";
        let sut = Markdown::parse(title);

        let result = sut.components().next().unwrap();

        assert_eq!(result, &Component::Heading1("Good bye"));
    }
}
