use std::iter::Peekable;
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

        let mut lines = input.lines().peekable();

        while let Some(line) = lines.peek() {
            if Markdown::is_skip(line) {
                // consume line
                let _ = lines.next().unwrap();
                continue;
            }
            if let Some(component) = Markdown::parse_heading1(line) {
                components.push(component);
                // consume line
                let _ = lines.next().unwrap();
                continue;
            }
            if let Some(component) = Markdown::parse_list(&mut lines) {
                components.push(component);
                continue;
            }
        }

        components
    }
    fn is_skip(line: &str) -> bool {
        line.is_empty()
    }
    fn parse_list(lines: &mut Peekable<Lines<'a>>) -> Option<Component<'a>> {
        let list = ItemList::parse(lines, 0);
        if list.item_len() > 0 {
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
    List(ItemList<'a>),
}
impl Component<'_> {
    fn from_line(line: &str) -> Component {
        Component::Heading1(line)
    }
}

#[derive(Debug, PartialEq)]
pub struct ItemList<'a> {
    items: Vec<Item<'a>>,
}
impl<'a> ItemList<'a> {
    const MARKS: [&'static str; 2] = ["- ", "* "];
    fn new() -> ItemList<'a> {
        ItemList { items: Vec::new() }
    }
    fn add(&mut self, item: Item<'a>) {
        self.items.push(item);
    }
    fn parse(lines: &mut Peekable<Lines<'a>>, indent: usize) -> Self {
        let mut result = Self::new();
        while let Some(line) = lines.peek() {
            if Self::is_skip(line) {
                let _ = lines.next().unwrap();
                continue;
            }
            if Self::is_end_loop(line) {
                return result;
            }
            // 指定されているインデントと同じ場合は同じ階層として追加
            if Self::is_item_line(line, indent) {
                let line = lines.next().unwrap();
                let mut item = Self::get_item_from_line(line, indent);
                // 子供がいれば再起的に子供を追加
                if let Some(child) = Self::get_children_from_line(lines, indent) {
                    item.add_child(child);
                }
                result.add(item);
                continue;
            }

            let indent_count = Self::indent_count(line);
            // 自分より親のインデントの場合は終了
            if indent_count < indent {
                return result;
            }
        }
        result
    }
    fn get_children_from_line(lines: &mut Peekable<Lines<'a>>, indent: usize) -> Option<Item<'a>> {
        while let Some(line) = lines.peek() {
            if Self::is_skip(line) {
                let _ = lines.next().unwrap();
                continue;
            }
            if Self::is_end_loop(line) {
                return None;
            }
            // インデントが同じ場合はNone
            if Self::is_item_line(line, indent) {
                return None;
            }

            let indent_count = Self::indent_count(line);

            // 自分より上位のインデントの場合は終了
            if indent_count < indent {
                return None;
            }
            // インデントが深くなった場合は子供として追加
            if indent < Self::indent_count(line) && Self::is_item_line(line, indent_count) {
                let line = lines.next().unwrap();
                let mut child = Self::get_item_from_line(line, indent_count);
                if let Some(grand_child) = Self::get_children_from_line(lines, indent_count) {
                    child.add_child(grand_child);
                }
                return Some(child);
            }
        }
        None
    }
    fn item_len(&self) -> usize {
        self.items.len()
    }
    fn is_item_line(line: &str, indent: usize) -> bool {
        line.starts_with(&Self::start_condition(indent))
    }
    fn indent_count(line: &str) -> usize {
        line.chars().take_while(|c| c == &' ').count()
    }
    fn is_skip(line: &str) -> bool {
        // 空行の場合はスキップ
        line.is_empty()
    }
    fn is_end_loop(line: &str) -> bool {
        // 空行ではないかつマークが含まれていない場合は終了
        !line.is_empty() && !Self::MARKS.iter().any(|mark| line.contains(mark))
    }
    fn start_condition(indent: usize) -> String {
        format!("{}{}", " ".repeat(indent), "- ")
    }
    fn get_item_from_line(line: &'a str, indent: usize) -> Item<'a> {
        let condition = Self::start_condition(indent);
        Item::new(line.trim_start_matches(&condition))
    }
}

#[derive(Debug, PartialEq)]
pub struct Item<'a> {
    value: &'a str,
    children: ItemList<'a>,
}
impl<'a> Item<'a> {
    fn new(value: &'a str) -> Self {
        Item {
            value,
            children: ItemList::new(),
        }
    }
    fn add_child(&mut self, item: Self) {
        self.children.add(item);
    }
}

#[derive(Debug, PartialEq)]
pub enum Heading<'a> {
    H1(&'a str),
    H2(&'a str),
    H3(&'a str),
    None(&'a str),
}
impl Heading<'_> {
    fn parse(line: &str) -> Heading {
        if line.starts_with("# ") {
            Heading::H1(&line[2..])
        } else if line.starts_with("## ") {
            Heading::H2(&line[3..])
        } else if line.starts_with("### ") {
            Heading::H3(&line[4..])
        } else {
            Heading::None(line)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn 複数の行をparseできる() {
        let mut lines = String::new();
        lines.push_str("# Hello World\n");
        lines.push_str("- foo\n");
        lines.push_str("\n");
        lines.push_str("    - bar\n");
        lines.push_str("# Good Bye\n");
        lines.push_str("- hoge\n");

        let sut = Markdown::parse(&lines);
        let mut sut = sut.components();

        let heading = sut.next().unwrap();
        assert_eq!(heading, &Component::Heading1("Hello World"));

        let list_foo = sut.next().unwrap();
        let mut list = Item::new("foo");
        list.add_child(Item::new("bar"));

        let mut expected = ItemList::new();
        expected.add(list);
        assert_eq!(list_foo, &Component::List(expected));

        let heading = sut.next().unwrap();
        assert_eq!(heading, &Component::Heading1("Good Bye"));

        let list_hoge = sut.next().unwrap();
        let mut list = Item::new("hoge");
        let mut expected = ItemList::new();
        expected.add(list);
        assert_eq!(list_hoge, &Component::List(expected));
    }

    // Only List tests
    mod list_test {
        use super::*;
        #[test]
        fn リスト以外の文字列までparseする() {
            let mut list = String::new();
            list.push_str("- foo\n");
            list.push_str("    - bar\n");
            list.push_str("         - hoge\n");
            list.push_str("\n");
            list.push_str("- chome\n");
            list.push_str(" - chome_child\n");
            list.push_str("# End of list\n");
            list.push_str("- foo\n");

            let mut list = list.lines().peekable();

            let sut = ItemList::parse(&mut list, 0);

            let grand_child = Item::new("hoge");

            let mut child = Item::new("bar");
            child.add_child(grand_child);

            let mut foo = Item::new("foo");
            foo.add_child(child);

            let mut chome = Item::new("chome");
            chome.add_child(Item::new("chome_child"));
            let mut expected = ItemList::new();
            expected.add(foo);
            expected.add(chome);

            assert_eq!(sut, expected);
        }
        #[test]
        fn リストは階層構造を持つ() {
            let mut list = String::new();
            list.push_str("- foo\n");
            list.push_str("    - bar\n");
            list.push_str("         - hoge\n");
            list.push_str("\n");
            list.push_str("- chome");
            let mut list = list.lines().peekable();

            let sut = ItemList::parse(&mut list, 0);

            let grand_child = Item::new("hoge");

            let mut child = Item::new("bar");
            child.add_child(grand_child);

            let mut foo = Item::new("foo");
            foo.add_child(child);

            let chome = Item::new("chome");

            let mut list = ItemList::new();
            list.add(foo);
            list.add(chome);

            assert_eq!(sut, list);
        }
        #[test]
        fn 文字列から単一のリストをparseできる() {
            let list = r#"- foo"#;
            let mut list = list.lines().peekable();
            let sut = ItemList::parse(&mut list, 0);

            let mut expected = ItemList::new();
            expected.add(Item::new("foo"));

            assert_eq!(sut, expected);
        }
    }
    mod heading_tests {
        use super::*;
        #[test]
        fn 文字列からタイトルをparseできる() {
            let title = "# Hello World";
            let result = Heading::parse(title);

            assert_eq!(result, Heading::H1("Hello World"));
        }
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
    #[test]
    fn lines_learning_test() {
        let lines = r#"
- foo"#;
        let mut lines = lines.lines();
        let first = lines.next().unwrap();
        assert_eq!(first, "");
        let second = lines.next().unwrap();
        assert_eq!(second, "- foo");
        let third = lines.next();
        assert_eq!(third, None);
    }
}
