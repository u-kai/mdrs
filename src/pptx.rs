use serde::{Deserialize, Serialize};

use crate::md::{Component, ItemList, Markdown, Page, Text};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pptx {
    filename: String,
    slides: Vec<Slide>,
}
impl Pptx {
    pub fn from_md_with_config(
        md: Markdown<'_>,
        filename: impl Into<String>,
        config: &ContentConfig,
    ) -> Self {
        let pages = md.pages();
        let slides = pages
            .into_iter()
            .map(|p| Slide::from_page_with_config(p, &config))
            .collect();
        Self {
            filename: filename.into(),
            slides,
        }
    }
    pub fn from_md(md: Markdown<'_>, filename: impl Into<String>) -> Self {
        let pages = md.pages();
        let slides = pages.into_iter().map(Slide::from).collect();
        Self {
            filename: filename.into(),
            slides,
        }
    }
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            slides: Vec::new(),
        }
    }
    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    r#type: String,
    title: Option<String>,
    contents: Vec<Content>,
}
impl Slide {
    fn from_page_with_config(page: Page<'_>, config: &ContentConfig) -> Self {
        let mut components = page.components();
        let component_num = page.components().count();
        if component_num == 0 {
            return Slide::blank();
        }
        if component_num == 1 {
            match components.next().unwrap() {
                Component::Text(Text::H1(title)) => {
                    return Slide::title_slide(*title);
                }
                Component::Text(text) => {
                    let mut result = Slide::blank();
                    result.add_content(Content::new(text.value()));
                    return result;
                }
                Component::SplitLine => {
                    return Slide::blank();
                }
                component @ _ => {
                    let mut result = Slide::blank();
                    let contents = Content::from_component_with_config(component, config);
                    contents.into_iter().for_each(|c| result.add_content(c));
                    return result;
                }
                _ => todo!(),
            }
        }

        fn components_to_contents(
            components: &[&Component<'_>],
            config: &ContentConfig,
        ) -> Vec<Content> {
            components
                .into_iter()
                .map(|c| Content::from_component_with_config(c, config))
                .flatten()
                .collect()
        }
        fn add_content_to_slide(slide: &mut Slide, content: Vec<Content>) {
            content.into_iter().for_each(|c| slide.add_content(c));
        }

        let first = components.next().unwrap();
        let mut slide = match first {
            Component::Text(Text::H1(title) | Text::H2(title) | Text::H3(title)) => {
                Slide::title_and_content(*title)
            }
            _ => {
                let mut result = Slide::blank();
                let contents = Content::from_component_with_config(first, config);
                add_content_to_slide(&mut result, contents);
                result
            }
        };
        let components = components.collect::<Vec<_>>();
        add_content_to_slide(
            &mut slide,
            components_to_contents(components.as_slice(), config),
        );
        slide
    }
    fn title_slide(title: impl Into<String>) -> Self {
        Self {
            r#type: "title_slide".to_string(),
            title: Some(title.into()),
            contents: Vec::new(),
        }
    }
    fn title_only(title: impl Into<String>) -> Self {
        Self {
            r#type: "title_only".to_string(),
            title: Some(title.into()),
            contents: Vec::new(),
        }
    }
    fn title_and_content(title: impl Into<String>) -> Self {
        Self {
            r#type: "title_and_content".to_string(),
            title: Some(title.into()),
            contents: Vec::new(),
        }
    }
    fn add_content(&mut self, content: Content) {
        self.contents.push(content);
    }
    fn blank() -> Self {
        Self {
            r#type: "blank".to_string(),
            title: None,
            contents: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Content {
    text: String,
    size: usize,
    bold: bool,
    children: Option<Vec<Content>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Font {
    pub size: usize,
    pub bold: bool,
}
impl Font {
    const H1_DEFAULT_SIZE: usize = 36;
    const H2_DEFAULT_SIZE: usize = 28;
    const H3_DEFAULT_SIZE: usize = 24;
    const NORMAL_SIZE: usize = 18;
    fn h1() -> Self {
        Self {
            size: Self::H1_DEFAULT_SIZE,
            bold: true,
        }
    }
    fn h2() -> Self {
        Self {
            size: Self::H2_DEFAULT_SIZE,
            bold: true,
        }
    }
    fn h3() -> Self {
        Self {
            size: Self::H3_DEFAULT_SIZE,
            bold: true,
        }
    }
    fn normal() -> Self {
        Self {
            size: Self::NORMAL_SIZE,
            bold: false,
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::normal()
    }
}

impl Content {
    fn from_font(text: impl Into<String>, font: Font) -> Self {
        Self {
            text: text.into(),
            children: None,
            size: font.size,
            bold: font.bold,
        }
    }
    fn new_with_font(text: impl Into<String>, font: Font) -> Self {
        Self::from_font(text, font)
    }
    fn to_bold(&mut self) {
        self.bold = true;
    }
    fn change_size(&mut self, size: usize) {
        self.size = size;
    }
    fn from_component_with_config(component: &Component<'_>, config: &ContentConfig) -> Vec<Self> {
        fn item_list_to_contents(
            item_list: &ItemList<'_>,
            config: &ContentConfig,
            level: usize,
        ) -> Vec<Content> {
            let mut result = vec![];
            for item in item_list.items() {
                let font = config.list_font(&item.value, level);
                let mut content = Content::new_with_font(item.value(), font);
                if item.children().items.len() == 0 {
                    result.push(content);
                    continue;
                }
                let children = item.children();
                content.children = Some(item_list_to_contents(children, config, level + 1));
                result.push(content);
            }
            result
        }
        fn text_to_content(text: &Text<'_>, config: &ContentConfig) -> Content {
            Content::from_font(text.value(), config.text_font(text))
        }
        match component {
            Component::List(list) => item_list_to_contents(list, &config, 0),
            Component::Text(text) => {
                vec![text_to_content(text, &config)]
            }
            _ => todo!(),
        }
    }
    fn from_component(component: &Component<'_>) -> Vec<Self> {
        fn item_list_to_contents(item_list: &ItemList<'_>) -> Vec<Content> {
            let mut result = vec![];
            for item in item_list.items() {
                let mut content = Content::new(item.value());
                if item.children().items.len() == 0 {
                    result.push(content);
                    continue;
                }
                let children = item.children();
                content.children = Some(item_list_to_contents(children));
                result.push(content);
            }
            result
        }
        match component {
            Component::List(list) => item_list_to_contents(list),
            Component::Text(text) => vec![Content::new(text.value())],
            _ => todo!(),
        }
    }
    fn new(text: impl Into<String>) -> Self {
        Self::from_font(text, Font::default())
    }
    fn add_child(&mut self, child: impl Into<String>) {
        if let Some(children) = &mut self.children {
            children.push(Content::new(child));
        } else {
            self.children = Some(vec![Content::new(child)]);
        }
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ContentConfig {
    h1: Font,
    h2: Font,
    h3: Font,
    normal: Font,
    per_level: usize,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            h1: Font::h1(),
            h2: Font::h2(),
            h3: Font::h3(),
            normal: Font::normal(),
            per_level: 4,
        }
    }
}
impl ContentConfig {
    fn list_font(&self, text: &Text<'_>, level: usize) -> Font {
        let mut font = self.text_font(text);
        font.size = font.size - (level * self.per_level);
        font
    }
    fn text_font(&self, text: &Text<'_>) -> Font {
        match text {
            Text::H1(_) => self.h1.clone(),
            Text::H2(_) => self.h2.clone(),
            Text::H3(_) => self.h3.clone(),
            Text::Normal(_) => self.normal.clone(),
        }
    }
    pub fn per_level(self, per_level: usize) -> Self {
        Self { per_level, ..self }
    }
    pub fn h1(self, font: Font) -> Self {
        Self { h1: font, ..self }
    }
    pub fn h2(self, font: Font) -> Self {
        Self { h2: font, ..self }
    }
    pub fn h3(self, font: Font) -> Self {
        Self { h3: font, ..self }
    }
    pub fn normal(self, font: Font) -> Self {
        Self {
            normal: font,
            ..self
        }
    }
    fn case_h1(&self) -> ContentConfigValue {
        ContentConfigValue {
            font: self.h1.clone(),
        }
    }
    fn case_h2(&self) -> ContentConfigValue {
        ContentConfigValue {
            font: self.h2.clone(),
        }
    }
    fn case_h3(&self) -> ContentConfigValue {
        ContentConfigValue {
            font: self.h3.clone(),
        }
    }
    fn case_normal(&self) -> ContentConfigValue {
        ContentConfigValue {
            font: self.normal.clone(),
        }
    }
}
struct ContentConfigValue {
    font: Font,
}

impl From<Page<'_>> for Slide {
    fn from(page: Page<'_>) -> Self {
        Self::from_page_with_config(page, &ContentConfig::default())
    }
}

#[cfg(test)]
mod tests {
    mod pptx_tests {
        use crate::{
            md::Markdown,
            pptx::{ContentConfig, Font, Pptx},
        };

        #[test]
        fn mdからpptxを作成可能() {
            let mut lines = String::new();
            lines.push_str("# Title\n");
            lines.push_str("---\n");
            lines.push_str("# Rust is very good language!!\n");
            lines.push_str("- So fast\n");
            lines.push_str("    - Because of no GC\n");
            lines.push_str("- So safe\n");
            lines.push_str("    - Because of borrow checker\n");
            lines.push_str("---\n");
            let md = Markdown::parse(&lines);
            let sut = Pptx::from_md(md, "test.pptx");

            assert_eq!(sut.slides.len(), 3);
        }
        #[test]
        fn configを設定可能() {
            let mut lines = String::new();
            lines.push_str("# Title\n");
            lines.push_str("---\n");
            lines.push_str("# Rust is very good language!!\n");
            lines.push_str("- # So fast\n");
            lines.push_str("    - Because of no GC\n");
            lines.push_str("- So safe\n");
            lines.push_str("    - Because of borrow checker\n");
            lines.push_str("---\n");
            let md = Markdown::parse(&lines);
            let config = ContentConfig::default().h1(Font {
                size: 100,
                bold: false,
            });
            let sut = Pptx::from_md_with_config(md, "test.pptx", &config);

            assert_eq!(sut.slides.len(), 3);
            assert_eq!(sut.slides[1].contents[0].size, 100);
            assert!(!sut.slides[1].contents[0].bold);
        }
    }
    mod slide_tests {
        use super::*;
        use crate::{
            md::{Component, Item, ItemList, Markdown, Page, Text},
            pptx::{ContentConfig, Font, Slide},
        };

        #[test]
        fn configを設定可能() {
            let config = ContentConfig::default().h1(Font {
                size: 100,
                bold: false,
            });

            let page = Page::new(&[
                Component::Text(Text::H1("Dummy")),
                Component::Text(Text::H1("Rust is very good language!!")),
            ]);
            let sut = Slide::from_page_with_config(page, &config);

            assert_eq!(sut.contents[0].size, 100);
            assert!(!sut.contents[0].bold);
        }
        #[test]
        fn pageの先頭要素がheadingでなければblankスライドを生成してcontentを追加する() {
            let text = Component::Text(Text::Normal("Rust is very good language!!"));
            let list = Component::List(ItemList {
                items: vec![
                    Item {
                        value: Text::H1("So fast"),
                        children: ItemList {
                            items: vec![Item {
                                value: Text::H1("Because of no GC"),
                                children: ItemList { items: vec![] },
                            }],
                        },
                    },
                    Item {
                        value: Text::H1("Nice type system"),
                        children: ItemList { items: vec![] },
                    },
                ],
            });
            let components = [text, list];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "blank");
            assert_eq!(sut.contents[0].text, "Rust is very good language!!");
            assert_eq!(sut.contents[1].text, "So fast");
            assert_eq!(
                sut.contents[1].children.as_ref().unwrap()[0].text,
                "Because of no GC"
            );
            assert_eq!(sut.contents[2].text, "Nice type system");
            assert_eq!(sut.contents[2].children.as_ref(), None);
        }
        #[test]
        fn pageの先頭要素がheadingでかつ他の要素があればtitle_and_contentスライドを生成してtitleとcontentを追加する(
        ) {
            let title_str = "Rust is very good language!!";
            let title = Component::Text(Text::H1(title_str));
            let content_str = "Rust is very good language!!";
            let content = Component::Text(Text::H2(content_str));
            let components = [title, content];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "title_and_content");
            assert_eq!(sut.title.unwrap(), title_str);
            assert_eq!(sut.contents[0].text, content_str);
        }
        #[test]
        fn pageの要素が一つかつその要素がheading1以外であればblankスライドを生成してcontentに追加する(
        ) {
            let content_str = "Rust is very good language!!";
            let content = Component::Text(Text::H2(content_str));
            let components = [content];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "blank");
            assert_eq!(sut.title, None);
            assert_eq!(sut.contents[0].text, content_str);
        }
        #[test]
        fn pageの要素が一つかつその要素がheading1であればtitleスライドを生成する() {
            let title_str = "Rust is very good language!!";
            let title = Component::Text(Text::H1(title_str));
            let components = [title];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "title_slide");
            assert_eq!(sut.title.unwrap(), title_str);
        }
        #[test]
        fn pageに要素が一つもなければblankスライドを生成する() {
            let page = Page::new(&[]);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "blank");
            assert_eq!(sut.title, None);
            assert_eq!(sut.contents.len(), 0);
        }
    }
    mod config_test {
        use crate::{
            md::{Component, Item, ItemList, Text},
            pptx::{Content, ContentConfig, Font},
        };
        #[test]
        fn configの設定は自由に変更できる_ver_text() {
            let config = ContentConfig::default()
                .h1(Font {
                    bold: true,
                    size: 32,
                })
                .h2(Font {
                    bold: false,
                    size: 100,
                })
                .h3(Font {
                    bold: true,
                    size: 110,
                })
                .normal(Font {
                    bold: true,
                    size: 180,
                });
            let component = Component::Text(Text::H1("Title"));
            let sut = Content::from_component_with_config(&component, &config);
            assert_eq!(sut[0].bold, true);
            assert_eq!(sut[0].size, 32);

            let component = Component::Text(Text::H2("Hello World"));
            let sut = Content::from_component_with_config(&component, &config);
            assert_eq!(sut[0].bold, false);
            assert_eq!(sut[0].size, 100);
            let component = Component::Text(Text::H3("Hello World"));
            let sut = Content::from_component_with_config(&component, &config);
            assert_eq!(sut[0].bold, true);
            assert_eq!(sut[0].size, 110);

            let component = Component::Text(Text::Normal("Hello World"));
            let sut = Content::from_component_with_config(&component, &config);
            assert_eq!(sut[0].bold, true);
            assert_eq!(sut[0].size, 180);
        }

        #[test]
        #[allow(non_snake_case)]
        fn ItemListのcontentのfontの低下値は変更可能() {
            let config = ContentConfig::default().per_level(10);
            let bottom = Item {
                value: Text::H1("Because of no GC!!"),
                children: ItemList { items: vec![] },
            };
            let middle = Item {
                value: Text::Normal("So fast!!"),
                children: ItemList {
                    items: vec![bottom],
                },
            };
            let top = Item {
                value: Text::Normal("Rust is very good language!!"),
                children: ItemList {
                    items: vec![middle],
                },
            };
            let component = Component::List(ItemList { items: vec![top] });
            let sut = Content::from_component_with_config(&component, &config);

            assert_eq!(sut[0].size, config.case_normal().font.size);
            assert_eq!(
                sut[0].children.as_ref().unwrap()[0].size,
                config.case_normal().font.size - 10
            );
            assert_eq!(
                sut[0].children.as_ref().unwrap()[0]
                    .children
                    .as_ref()
                    .unwrap()[0]
                    .size,
                config.case_h1().font.size - 20
            );
            assert_eq!(
                sut[0].children.as_ref().unwrap()[0]
                    .children
                    .as_ref()
                    .unwrap()[0]
                    .bold,
                config.case_h1().font.bold
            );
        }
        #[test]
        #[allow(non_snake_case)]
        fn ItemListのcontentのfontは下層に降るほどfontが小さくなる() {
            let config = ContentConfig::default();
            let bottom = Item {
                value: Text::H1("Because of no GC!!"),
                children: ItemList { items: vec![] },
            };
            let middle = Item {
                value: Text::Normal("So fast!!"),
                children: ItemList {
                    items: vec![bottom],
                },
            };
            let top = Item {
                value: Text::Normal("Rust is very good language!!"),
                children: ItemList {
                    items: vec![middle],
                },
            };
            let component = Component::List(ItemList { items: vec![top] });
            let sut = Content::from_component_with_config(&component, &config);

            assert_eq!(sut[0].size, config.case_normal().font.size);
            assert!(sut[0].children.as_ref().unwrap()[0].size < config.case_normal().font.size);
            assert!(
                sut[0].children.as_ref().unwrap()[0]
                    .children
                    .as_ref()
                    .unwrap()[0]
                    .size
                    < config.case_h1().font.size
            );
            assert_eq!(
                sut[0].children.as_ref().unwrap()[0]
                    .children
                    .as_ref()
                    .unwrap()[0]
                    .bold,
                config.case_h1().font.bold
            );
        }
        #[test]
        #[allow(non_snake_case)]
        fn contentのfontの設定をTextの列挙子によって切り分ける() {
            let config = ContentConfig::default();
            let component = Component::Text(Text::H1("Title"));
            let sut = Content::from_component_with_config(&component, &config);

            assert_eq!(sut[0].bold, config.case_h1().font.bold);
            assert_eq!(sut[0].size, config.case_h1().font.size);

            let config = ContentConfig::default();
            let component = Component::Text(Text::H2("Hello World"));
            let sut = Content::from_component_with_config(&component, &config);

            assert_eq!(sut[0].bold, config.case_h2().font.bold);
            assert_eq!(sut[0].size, config.case_h2().font.size);

            let config = ContentConfig::default();
            let component = Component::Text(Text::Normal("Hello World"));
            let sut = Content::from_component_with_config(&component, &config);

            assert_eq!(sut[0].bold, config.case_normal().font.bold);
            assert_eq!(sut[0].size, config.case_normal().font.size);
        }
    }

    mod content_test {
        use crate::{
            md::{Component, Item, ItemList, Text},
            pptx::Content,
        };

        #[test]
        fn contentの初期fontはサイズが18でboldではない() {
            let sut = Content::new("Hello World");

            assert_eq!(sut.size, 18);
            assert!(!sut.bold);
        }
        #[test]
        fn contentはfontの設定が可能() {
            let mut sut = Content::new("Hello World");

            sut.change_size(28);
            sut.to_bold();

            assert_eq!(sut.size, 28);
            assert!(sut.bold);
        }
        #[test]
        #[allow(non_snake_case)]
        fn contentはComponentのTextから生成できる() {
            let component = Component::Text(Text::H2("Hello World"));

            let sut = Content::from_component(&component);

            assert_eq!(sut[0].text, "Hello World");
        }
        #[test]
        #[allow(non_snake_case)]
        fn contentはComponentのListから生成できる() {
            // - Root1
            //  - Parent1
            // - Root2
            //  - Parent2
            //
            let list = ItemList {
                items: vec![
                    Item {
                        value: Text::H2("Root1"),
                        children: ItemList {
                            items: vec![Item {
                                value: Text::Normal("Parent1"),
                                children: ItemList { items: vec![] },
                            }],
                        },
                    },
                    Item {
                        value: Text::H2("Root2"),
                        children: ItemList {
                            items: vec![Item {
                                value: Text::Normal("Parent2"),
                                children: ItemList { items: vec![] },
                            }],
                        },
                    },
                ],
            };

            let component = Component::List(list);

            let sut = Content::from_component(&component);

            assert_eq!(sut[0].text, "Root1");
            assert_eq!(sut[0].children.as_ref().unwrap()[0].text, "Parent1");
            assert_eq!(sut[1].text, "Root2");
            assert_eq!(sut[1].children.as_ref().unwrap()[0].text, "Parent2");
        }
    }
}
