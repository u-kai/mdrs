use std::str::Lines;

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
            let list_str = line.trim_start_matches("- ");
            let mut list = List::new();
            list.add(list_str);
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
    List(List<'a>),
}

#[derive(Debug, PartialEq)]
pub struct List<'a> {
    items: Vec<&'a str>,
    children: Vec<List<'a>>,
}
impl<'a> List<'a> {
    const MARKS: &'static [&'static str] = &["- ", "* "];
    fn new() -> List<'a> {
        List {
            items: Vec::new(),
            children: Vec::new(),
        }
    }
    fn parse(list: &mut Lines<'a>, indent: usize) -> Self {
        let mut result = List::new();
        while let Some(line) = list.next() {
            if Self::is_end_loop(line) {
                break;
            }
            // インデントが同じ場合は同じ階層として追加
            if Self::is_item_line(line, indent) {
                result.add(Self::get_item_from_line(line, indent));
                continue;
            }
            let indent_count = Self::indent_count(line);
            // インデントが深くなった場合は子供として追加
            if indent < Self::indent_count(line) && Self::is_item_line(line, indent_count) {
                let mut child = List::new();
                child.add(Self::get_item_from_line(line, indent_count));
                let rest = List::parse(list, indent_count);
                child.concat(rest);
                result.add_child(child);
                continue;
            }
        }
        result
    }
    fn is_item_line(line: &str, indent: usize) -> bool {
        line.starts_with(&Self::start_condition(indent))
    }
    fn indent_count(line: &str) -> usize {
        line.chars().take_while(|c| c == &' ').count()
    }
    fn is_end_loop(line: &str) -> bool {
        // 空行ではないかつマークが含まれていない場合は終了
        !line.is_empty() && !Self::MARKS.iter().any(|mark| line.contains(mark))
    }
    fn start_condition(indent: usize) -> String {
        format!("{}{}", " ".repeat(indent), "- ")
    }
    fn get_item_from_line(line: &'a str, indent: usize) -> &'a str {
        let condition = Self::start_condition(indent);
        line.trim_start_matches(&condition)
    }
    fn concat(&mut self, other: Self) {
        let (items, children) = (other.items, other.children);
        for item in items {
            self.add(item);
        }
        for rest_child in children {
            self.add_child(rest_child);
        }
    }
    fn add(&mut self, item: &'a str) {
        self.items.push(item);
    }
    fn add_child(&mut self, child: List<'a>) {
        self.children.push(child);
    }
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
        let mut list = List::new();
        list.add("foo");
        assert_eq!(list_foo, &Component::List(list));
    }
    #[test]
    fn 文字列からリストをparseできる() {
        let list = r#"- foo"#;
        let sut = Markdown::parse(list);
        let mut sut = sut.components();

        let list_foo = sut.next().unwrap();
        let mut list = List::new();
        list.add("foo");

        assert_eq!(list_foo, &Component::List(list));
    }
    #[test]
    fn リストは階層構造を持つ() {
        let list = r#"
- foo
    - bar

        - hoge"#;
        let mut list = list.lines();
        let sut = List::parse(&mut list, 0);

        let mut list = List::new();
        list.add("foo");
        let mut child = List::new();
        child.add("bar");
        let mut grand_child = List::new();
        grand_child.add("hoge");
        child.add_child(grand_child);
        list.add_child(child);

        assert_eq!(sut, list);
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
