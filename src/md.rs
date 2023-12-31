use std::iter::Peekable;
use std::str::Lines;

#[derive(Debug, PartialEq)]
pub struct Markdown<'a> {
    components: Vec<Component<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Page<'a> {
    components: &'a [Component<'a>],
}

impl<'a> Page<'a> {
    pub fn new(components: &'a [Component<'a>]) -> Self {
        Self { components }
    }
    pub fn components(&self) -> impl Iterator<Item = &'a Component<'a>> {
        self.components.iter()
    }
}
impl<'a> Markdown<'a> {
    pub fn parse(input: &'a str) -> Markdown {
        let components = Markdown::parse_components(input);
        Markdown { components }
    }
    pub fn pages(&'a self) -> impl Iterator<Item = Page<'a>> {
        self.components
            .split(|c| c == &Component::SplitLine)
            .map(|c| Page::new(c))
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

            if let Some(_split_line) = SplitLine::parse(line) {
                components.push(Component::SplitLine);
                // consume line
                let _ = lines.next().unwrap();
                continue;
            }

            if ItemList::is_item_list_line(line) {
                if let Some(component) = Markdown::parse_list(&mut lines) {
                    components.push(component);
                    continue;
                }
            }
            // それ以外の場合はテキストとして追加
            let line = lines.next().unwrap();
            components.push(Markdown::parse_text(line));
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
    fn parse_text(line: &'a str) -> Component<'a> {
        Component::Text(Text::parse(line))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Component<'a> {
    Text(Text<'a>),
    List(ItemList<'a>),
    SplitLine,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemList<'a> {
    pub(crate) items: Vec<Item<'a>>,
}
impl<'a> ItemList<'a> {
    const MARKS: [&'static str; 2] = ["- ", "* "];

    fn new() -> ItemList<'a> {
        ItemList { items: Vec::new() }
    }
    fn add_item(&mut self, item: Item<'a>) {
        self.items.push(item);
    }
    fn add_child(&mut self, children: Self) {
        if let Some(parent) = self.items.last_mut() {
            children.items.into_iter().for_each(|child| {
                parent.add_child(child);
            })
        };
    }
    fn add_sibling(&mut self, sibling: Self) {
        sibling
            .items
            .into_iter()
            .for_each(|sibling_item| self.add_item(sibling_item))
    }
    fn parse(lines: &mut Peekable<Lines<'a>>, mut indent: usize) -> Self {
        let mut result = Self::new();
        while let Some(line) = lines.peek() {
            if Self::is_skip(line) {
                let _ = lines.next().unwrap();
                continue;
            }
            // list line以外の場合はlineを消費せずに終了する
            if !Self::is_item_list_line(line) {
                return result;
            }
            // 自分より親のインデントの場合はlineを消費せずに終了
            if Self::is_parent_indent(line, indent) {
                return result;
            }
            // 指定されているインデントと同じ場合は同じ階層として追加
            if Self::is_same_indent(line, indent) {
                let line = lines.next().unwrap();
                let mut sibling = Self::from_line(line, indent);
                let children = Self::parse_children(lines, indent);
                sibling.add_child(children);

                result.add_sibling(sibling);
                continue;
            }

            // 自分より子のインデントの場合は再起的に子供を追加
            if Self::is_children_indent(line, indent) {
                let indent_count = Self::indent_count(line);
                // そもそもresultにまだitemが存在しなければ当該indentが最初のitemになり，同じindentの要素をparseするようにする
                if result.item_len() == 0 {
                    return Self::parse(lines, indent_count);
                }
                let line = lines.next().unwrap();
                let mut children = Self::from_line(line, indent_count);
                children.add_child(Self::parse(lines, indent_count));
                result.add_child(children);
            }
        }
        result
    }
    fn parse_children(lines: &mut Peekable<Lines<'a>>, indent: usize) -> Self {
        Self::parse(lines, indent + 1)
    }
    fn is_skip(line: &str) -> bool {
        // 空行の場合はスキップ
        line.is_empty()
    }
    fn is_same_indent(line: &str, indent: usize) -> bool {
        line.starts_with(&Self::start_condition(indent))
    }
    fn is_parent_indent(line: &str, indent: usize) -> bool {
        let indent_count = Self::indent_count(line);
        indent_count < indent
    }
    fn is_children_indent(line: &str, indent: usize) -> bool {
        let indent_count = Self::indent_count(line);
        indent_count > indent
    }
    fn indent_count(line: &str) -> usize {
        line.chars().take_while(|c| c == &' ').count()
    }
    fn is_item_list_line(line: &str) -> bool {
        let first_str = line.trim_start().get(0..2);
        if let Some(first_str) = first_str {
            ItemList::MARKS.iter().any(|s| *s == first_str)
        } else {
            false
        }
    }
    fn start_condition(indent: usize) -> String {
        format!("{}{}", " ".repeat(indent), "- ")
    }
    fn from_line(line: &'a str, indent: usize) -> Self {
        let condition = Self::start_condition(indent);
        Self {
            items: vec![Item::new(line.trim_start_matches(&condition))],
        }
    }
    pub fn items(&'a self) -> impl Iterator<Item = &'a Item<'a>> {
        self.items.iter()
    }
    fn item_len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item<'a> {
    pub(crate) value: Text<'a>,
    pub(crate) children: ItemList<'a>,
}
impl<'a> Item<'a> {
    pub fn children(&'a self) -> &ItemList<'a> {
        &self.children
    }
    pub fn value(&self) -> &str {
        self.value.value()
    }
    fn new(value: &'a str) -> Self {
        Item {
            value: Text::parse(value),
            children: ItemList::new(),
        }
    }
    fn add_child(&mut self, item: Self) {
        self.children.add_item(item);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Text<'a> {
    H1(&'a str),
    H2(&'a str),
    H3(&'a str),
    Normal(&'a str),
}
impl Text<'_> {
    pub fn value(&self) -> &str {
        match self {
            Text::H1(value) => value,
            Text::H2(value) => value,
            Text::H3(value) => value,
            Text::Normal(value) => value,
        }
    }
    fn parse(line: &str) -> Text {
        if line.starts_with("# ") {
            return Text::H1(&line[2..]);
        }
        if line.starts_with("## ") {
            return Text::H2(&line[3..]);
        }
        if line.starts_with("### ") {
            return Text::H3(&line[4..]);
        }
        let hash_count = line.chars().take_while(|c| c == &'#').count();
        if hash_count > 3 && &line[hash_count..hash_count + 1] == " " {
            return Text::H3(&line[hash_count + 1..]);
        }
        Text::Normal(line)
    }
}
#[derive(Debug, PartialEq)]
pub struct SplitLine;
impl SplitLine {
    fn parse(line: &str) -> Option<Self> {
        if line == "---" || line == "***" || line == "---\n" || line == "***\n" {
            Some(SplitLine)
        } else {
            None
        }
    }
    fn to_str(&self) -> &str {
        "---"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn 改行文字がなければ一つの行として評価する() {
        let mut lines = String::new();
        lines.push_str("# Title");
        lines.push_str("---");
        lines.push_str("# Rust is very good language!!");
        lines.push_str("- So fast");
        lines.push_str("    - Because of no GC");
        lines.push_str("- So safe");
        lines.push_str("    - Because of borrow checker");
        lines.push_str("---");
        let sut = Markdown::parse(&lines);

        let mut sut = sut.components();
        let heading = sut.next().unwrap();

        assert_eq!(heading, &Component::Text(Text::H1("Title---# Rust is very good language!!- So fast    - Because of no GC- So safe    - Because of borrow checker---")));
    }

    #[test]
    fn learning_loop() {
        let data = r#"---

# 目的

- TDD は何なのかがわかるようになる

  - 何を目的としているものなのか？
  - どのような手法なのか？
  - 何が嬉しいのか？

TDD が必要な理由/背景がわかる

  - 何を目的としているものなのか？
  - どのような手法なのか？
  - 何が嬉しいのか？

- TDD は何なのかがわかるようになる

"#;
        let md = Markdown::parse(data);
        let data = r#"---

# 目的

- TDD は何なのかがわかるようになる

  - 何を目的としているものなのか？
  - どのような手法なのか？
  - 何が嬉しいのか？

- TDD が必要な理由/背景がわかる
"#;
        let md = Markdown::parse(data);
        let data = r#"---

# 良いテストの考え方

一般的に良いテストとは次のような考え方

### テスト実行方法
        "#;
        let md = Markdown::parse(data);
    }

    #[test]
    fn 複数の行をparseできる() {
        let mut lines = String::new();
        lines.push_str("# Hello World\n");
        lines.push_str("- foo\n");
        lines.push_str("\n");
        lines.push_str("    - bar\n");
        lines.push_str("---\n");
        lines.push_str("# Good Bye\n");
        lines.push_str("- hoge\n");

        let sut = Markdown::parse(&lines);
        let mut sut = sut.components();

        let heading = sut.next().unwrap();
        assert_eq!(heading, &Component::Text(Text::H1("Hello World")));

        let list_foo = sut.next().unwrap();
        let mut list = Item::new("foo");
        list.add_child(Item::new("bar"));
        let mut expected = ItemList::new();
        expected.add_item(list);
        assert_eq!(list_foo, &Component::List(expected));

        let split = sut.next().unwrap();
        assert_eq!(split, &Component::SplitLine);

        let heading = sut.next().unwrap();
        assert_eq!(heading, &Component::Text(Text::H1("Good Bye")));

        let list_hoge = sut.next().unwrap();
        let mut expected = ItemList::new();
        expected.add_item(Item::new("hoge"));
        assert_eq!(list_hoge, &Component::List(expected));
    }
    #[test]
    fn splitを境にpage構造体を作成することができる() {
        let title_page_component = Component::Text(Text::H1("Learn Rust"));
        let describe_page_title = Component::Text(Text::H1("Why Rust is very popular?"));
        let describe_page_list = Component::List(ItemList {
            items: vec![
                Item {
                    value: Text::H3("So fast"),
                    children: ItemList {
                        items: vec![Item {
                            value: Text::Normal("Rust has not GC"),
                            children: ItemList { items: vec![] },
                        }],
                    },
                },
                Item {
                    value: Text::H3("So readable!"),
                    children: ItemList { items: vec![] },
                },
            ],
        });
        let sut = Markdown {
            components: vec![
                title_page_component.clone(),
                Component::SplitLine,
                describe_page_title.clone(),
                describe_page_list.clone(),
            ],
        };

        let mut pages = sut.pages();
        let mut title_page = pages.next().unwrap().components();
        let title_component = title_page.next().unwrap();
        assert_eq!(title_component, &title_page_component);
        assert_eq!(title_page.next(), None);

        let mut describe_page = pages.next().unwrap().components();
        let describe_component = describe_page.next().unwrap();
        assert_eq!(describe_component, &describe_page_title);
        let describe_component = describe_page.next().unwrap();
        assert_eq!(describe_component, &describe_page_list);
        assert_eq!(describe_page.next(), None);

        assert_eq!(pages.next(), None);
    }
    #[test]
    fn split_lineで終了している場合はcomponentsが空のpageが最後に生成される() {
        let title_page_component = Component::Text(Text::H1("Learn Rust"));
        let sut = Markdown {
            components: vec![title_page_component.clone(), Component::SplitLine],
        };

        let mut pages = sut.pages();
        let mut title_page = pages.next().unwrap().components();
        let title_component = title_page.next().unwrap();
        assert_eq!(title_component, &title_page_component);
        assert_eq!(title_page.next(), None);
        assert_eq!(pages.next().unwrap(), Page { components: &[] });
        assert_eq!(pages.next(), None);
    }

    // Only List tests
    mod list_test {
        use super::*;
        #[test]
        fn リスト内のheadingを考慮できる() {
            let list = r#"- # foo"#;
            let mut list = list.lines().peekable();
            let sut = ItemList::parse(&mut list, 0);

            let mut expected = ItemList::new();
            expected.add_item(Item::new("# foo"));

            assert_eq!(sut.items[0].value, Text::H1("foo"));
            assert_eq!(sut, expected);
        }
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
            expected.add_item(foo);
            expected.add_item(chome);

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
            list.add_item(foo);
            list.add_item(chome);

            assert_eq!(sut, list);
        }
        #[test]
        fn 兄弟を持つリストをparseできる() {
            let list = r#"
- foo
- bar
- hoge"#;
            let mut list = list.lines().peekable();
            let sut = ItemList::parse(&mut list, 0);

            let mut expected = ItemList::new();
            expected.add_item(Item::new("foo"));
            expected.add_item(Item::new("bar"));
            expected.add_item(Item::new("hoge"));

            assert_eq!(sut, expected);
        }
        #[test]
        fn 文字列から単一のリストをparseできる() {
            let list = r#"- foo"#;
            let mut list = list.lines().peekable();
            let sut = ItemList::parse(&mut list, 0);

            let mut expected = ItemList::new();
            expected.add_item(Item::new("foo"));

            assert_eq!(sut, expected);
        }
    }
    mod heading_tests {
        use super::*;
        #[test]
        fn 何もない文字列をparseできる() {
            let title = "Normal";
            let result = Text::parse(title);

            assert_eq!(result, Text::Normal("Normal"));
        }
        #[test]
        fn 文字列からタイトルをparseできる() {
            let title = "# Hello World";
            let result = Text::parse(title);

            assert_eq!(result, Text::H1("Hello World"));
        }
        #[test]
        fn 文字列からh2をparseできる() {
            let title = "## Hello World";
            let result = Text::parse(title);

            assert_eq!(result, Text::H2("Hello World"));
        }
        #[test]
        fn 文字列からマークが3以上はh3としてparseできる() {
            let title = "#### Hello World";
            let result = Text::parse(title);
            assert_eq!(result, Text::H3("Hello World"));
        }
    }
    mod split_tests {
        use super::*;

        #[test]
        fn splitをparseできる() {
            let split = "---";
            let result = SplitLine::parse(split);
            assert_eq!(result, Some(SplitLine))
        }
        #[test]
        fn 改行されるsplitをparseできる() {
            let split = "---\n";
            let result = SplitLine::parse(split);
            assert_eq!(result, Some(SplitLine))
        }
        #[test]
        fn splitは文字列に変換できる() {
            let sut = SplitLine::parse("---").unwrap();

            assert_eq!(sut.to_str(), "---");
        }
    }
    #[test]
    fn learning_改行文字がない時のlinesの挙動を確認() {
        let mut lines = "foo".lines().peekable();
        assert_eq!(lines.next(), Some("foo"));
        assert_eq!(lines.next(), None);
    }
}
