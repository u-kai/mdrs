use serde::{Deserialize, Serialize};

use crate::md::{Component, Markdown, Page, Text};

//fn page_to_slide(page: Vec<Component>) -> Slide {
//    let mut result = Slide::blank();
//    for (i, component) in page.into_iter().enumerate() {
//        match component {
//            Component::List(list) => {
//                for item in list.items() {
//                    let mut content = Content::new(item.value());
//                    for child in item.children() {
//                        content.add_child(child.value());
//                    }
//                }
//            }
//            Component::SplitLine => panic!("SplitLine is not allowed in page"),
//            Component::Text(text) if i == 0 => match text {
//                Text::Normal(text) => {
//                    result.add_content(Content::new(text));
//                }
//                text => {
//                    result = Slide::title_only(text.value().to_string());
//                }
//            },
//            Component::Text(text) => match text {
//                Text::Normal(text) => {
//                    result.add_content(Content::new(text));
//                }
//                _ => {
//                    result.add_content(Content::new(text.value()));
//                }
//            },
//        }
//    }
//    result
//}
//fn md_to_slides(md: Markdown<'_>) -> Vec<Slide> {
//    let mut result = Vec::new();
//    let pages = md.components().cloned().collect::<Vec<_>>();
//    let mut pages = pages.split(|c| c == &Component::SplitLine);
//
//    let init = pages.next().unwrap();
//    match init {
//        Component::Text(Text::H1(title)) => {
//            result.push(Slide::title_only(*title));
//        }
//        _ => {}
//    }
//    for page in pages {
//        match
//    }
//    result
//}

fn md_to_pptx(md: Markdown<'_>, filename: impl Into<String>) -> Pptx {
    let mut result = Pptx::new(filename);
    let mut page_num = 0;
    for component in md.components() {}

    result
}

impl Pptx {
    pub fn from_md(md: Markdown<'_>, filename: impl Into<String>) -> Self {
        md_to_pptx(md, filename)
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
pub struct Pptx {
    filename: String,
    slides: Vec<Slide>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    r#type: String,
    title: Option<String>,
    content: Vec<Content>,
}
impl Slide {
    fn title_only(title: impl Into<String>) -> Self {
        Self {
            r#type: "title_only".to_string(),
            title: Some(title.into()),
            content: Vec::new(),
        }
    }
    fn title_and_content(title: impl Into<String>) -> Self {
        Self {
            r#type: "title_and_content".to_string(),
            title: Some(title.into()),
            content: Vec::new(),
        }
    }
    fn add_content(&mut self, content: Content) {
        self.content.push(content);
    }
    fn blank() -> Self {
        Self {
            r#type: "blank".to_string(),
            title: None,
            content: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Content {
    text: String,
    children: Option<Vec<Content>>,
}

impl Content {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            children: None,
        }
    }
    fn add_child(&mut self, child: impl Into<String>) {
        if let Some(children) = &mut self.children {
            children.push(Content::new(child));
        } else {
            self.children = Some(vec![Content::new(child)]);
        }
    }
}

impl From<Page<'_>> for Slide {
    fn from(page: Page<'_>) -> Self {
        let mut components = page.components();
        let component_num = page.components().count();

        if component_num == 1 {
            match components.next().unwrap() {
                Component::Text(Text::H1(title)) => {
                    return Slide::title_only(*title);
                }
                Component::Text(text) => {
                    let mut result = Slide::blank();
                    result.add_content(Content::new(text.value()));
                    return result;
                }
                _ => todo!(),
            }
        }
        let mut result = Slide::blank();
        for component in components {
            match component {
                Component::Text(text) => {
                    result.add_content(Content::new(text.value()));
                }
                _ => todo!(),
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    mod slide_test {
        use super::*;
        use crate::{
            md::{Component, Markdown, Page, Text},
            pptx::Slide,
        };
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
            assert_eq!(sut.content[0].text, content_str);
        }
        #[test]
        fn pageの要素が一つかつその要素がheading1であればtitleスライドを生成する() {
            let title_str = "Rust is very good language!!";
            let title = Component::Text(Text::H1(title_str));
            let components = [title];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.r#type, "title_only");
            assert_eq!(sut.title.unwrap(), title_str);
        }
        #[test]
        fn slideはpageから生成可能() {
            let components1 = Component::Text(Text::H2("Hello World"));
            let components2 = Component::Text(Text::H2("Good Bye"));
            let components = [components1, components2];
            let page = Page::new(&components);

            let sut = Slide::from(page);

            assert_eq!(sut.content[0].text, "Hello World");
            assert_eq!(sut.content[1].text, "Good Bye");
        }
    }

    //#[test]
    //fn md_をpptxの構造体に変換する() {
    //    let mut md = String::new();
    //    md.push_str("# Title\n");
    //    md.push_str("---");
    //    md.push_str("# Languages\n");
    //    md.push_str("- Rust\n");
    //    md.push_str("   - Very fast\n");
    //    md.push_str("- Python\n");
    //    md.push_str("   - Very popular\n");
    //    md.push_str("---");
    //    let md = Markdown::parse(&md);
    //    let pptx = Pptx::from_md(md, "test.pptx");

    //    let mut expected = Pptx::new("test.pptx");
    //    expected.add_slide(Slide::title_only("Title"));
    //    let mut title_and_content = Slide::title_and_content("Languages");
    //    let mut rust = Content::new("Rust");
    //    rust.add_child("Very fast");
    //    let mut python = Content::new("Python");
    //    python.add_child("Very popular");
    //    title_and_content.add_content(rust);
    //    title_and_content.add_content(python);
    //    expected.add_slide(title_and_content);

    //    assert_eq!(pptx, expected);
    //}
}
