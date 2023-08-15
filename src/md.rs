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
            if let Some(component) = Markdown::parse_heading1(line) {
                components.push(component);
            }
            if let Some(component) = Markdown::parse_list(line) {
                components.push(component);
            }
        }
        components
    }
    fn parse_list(line: &'a str) -> Option<Component<'a>> {
        if line.starts_with("- ") {
            let list = line.trim_start_matches("- ");
            Some(Component::List(list))
        } else {
            None
        }
    }
    fn parse_heading1(line: &'a str) -> Option<Component<'a>> {
        if line.starts_with("# ") {
            let heading = line.trim_start_matches("# ");
            Some(Component::Heading1(heading))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Component<'a> {
    Heading1(&'a str),
    List(&'a str),
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
        let mut sut = sut.components();

        let heading = sut.next().unwrap();

        assert_eq!(heading, &Component::Heading1("Hello World"));

        let list_foo = sut.next().unwrap();
        assert_eq!(list_foo, &Component::List("foo"));
    }
    #[test]
    fn 文字列からリストをparseできる() {
        let list = r#"- foo"#;
        let sut = Markdown::parse(list);
        let mut sut = sut.components();

        let list_foo = sut.next().unwrap();

        assert_eq!(list_foo, &Component::List("foo"));
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
