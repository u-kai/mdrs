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
        let list = List::parse(lines, 0);
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
    List(List<'a>),
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
        // 空行ではないかつマークが含まれていない場合は終了
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
    fn concat(&mut self, other: Self) {
        self.items.extend(other.items);
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
pub struct List<'a> {
    items: Vec<&'a str>,
    children: Vec<List<'a>>,
}
impl<'a> List<'a> {
    // todo マークの種類を増やす
    const MARKS: &'static [&'static str] = &["- ", "* "];
    fn new() -> List<'a> {
        List {
            items: Vec::new(),
            children: Vec::new(),
        }
    }
    fn parse(lines: &mut Peekable<Lines<'a>>, indent: usize) -> Self {
        let mut result = List::new();
        while let Some(line) = lines.peek() {
            if Self::is_skip(line) {
                let _ = lines.next().unwrap();
                continue;
            }
            if Self::is_end_loop(line) {
                return result;
            }
            // インデントが同じ場合は同じ階層として追加
            if Self::is_item_line(line, indent) {
                let line = lines.next().unwrap();
                result.add(Self::get_item_from_line(line, indent));
                continue;
            }

            let indent_count = Self::indent_count(line);

            // 自分より上位のインデントの場合は終了
            if indent_count < indent {
                return result;
            }
            // インデントが深くなった場合は子供として追加
            if indent < Self::indent_count(line) && Self::is_item_line(line, indent_count) {
                let mut child = List::new();
                let line = lines.next().unwrap();
                child.add(Self::get_item_from_line(line, indent_count));
                let rest = List::parse(lines, indent_count);
                child.concat(rest);
                result.add_child(child);
                continue;
            }
        }
        result
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
        // 空行ではないかつマークが含まれていない場合は終了
        line.is_empty()
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
    //#[test]
    //fn 複数の行をparseできる() {
    //    let mut lines = String::new();
    //    lines.push_str("# Hello World\n");
    //    lines.push_str("- foo\n");
    //    lines.push_str("\n");
    //    lines.push_str("    - bar\n");
    //    lines.push_str("# Good Bye\n");

    //    let sut = Markdown::parse(&lines);
    //    let mut sut = sut.components();

    //    let heading = sut.next().unwrap();
    //    assert_eq!(heading, &Component::Heading1("Hello World"));

    //    let list_foo = sut.next().unwrap();
    //    let mut list = List::new();
    //    list.add("foo");
    //    let mut bar = List::new();
    //    bar.add("bar");
    //    list.add_child(bar);
    //    assert_eq!(list_foo, &Component::List(list));

    //    let heading = sut.next().unwrap();
    //    assert_eq!(heading, &Component::Heading1("Good Bye"));
    //}

    // Only List tests
    mod list_test {
        use super::*;
        // #[test]
        //        fn リスト以外の文字列までparseする() {
        //            let mut list = String::new();
        //            list.push_str("- foo\n");
        //            list.push_str("    - bar\n");
        //            list.push_str("         - hoge\n");
        //            list.push_str("\n");
        //            list.push_str("- chome");
        //            list.push_str(" - chome_child");
        //            list.push_str("# End of list");
        //            list.push_str("- foo\n");
        //
        //            let mut list = list.lines().peekable();
        //
        //            let sut = List::parse(&mut list, 0);
        //
        //            let mut grand_child = List::new();
        //            grand_child.add("hoge");
        //
        //            let mut child = List::new();
        //            child.add("bar");
        //            child.add_child(grand_child);
        //
        //            let mut list = List::new();
        //            list.add("foo");
        //            list.add("chome");
        //            list.add_child(child);
        //
        //            assert_eq!(sut, list);
        //        }
        //#[test]
        //fn リストは階層構造を持つ() {
        //    let mut list = String::new();
        //    list.push_str("- foo\n");
        //    list.push_str("    - bar\n");
        //    list.push_str("         - hoge\n");
        //    list.push_str("\n");
        //    list.push_str("- chome");
        //    let mut list = list.lines().peekable();

        //    let sut = List::parse(&mut list, 0);

        //    let mut grand_child = List::new();
        //    grand_child.add("hoge");

        //    let mut child = List::new();
        //    child.add("bar");
        //    child.add_child(grand_child);

        //    let mut list = List::new();
        //    list.add("foo");
        //    list.add("chome");
        //    list.add_child(child);

        //    assert_eq!(sut, list);
        //}
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
