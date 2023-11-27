#[derive(Debug, PartialEq, Eq)]
struct ActionTree {
    name: String,
    input: Vec<ActionInput>,
    output: Vec<ActionOutput>,
    children: Vec<ActionTree>,
}

impl ActionTree {
    fn root(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input: Vec::new(),
            output: Vec::new(),
            children: Vec::new(),
        }
    }
    fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input: Vec::new(),
            output: Vec::new(),
            children: Vec::new(),
        }
    }
    fn add_input(mut self, name: &str, value: Box<dyn ToJson>) -> Self {
        self.input.push(ActionInput {
            name: name.to_string(),
            value,
        });
        self
    }
    fn add_output(mut self, name: &str, value: Box<dyn ToJson>) -> Self {
        self.output.push(ActionOutput {
            name: name.to_string(),
            value,
        });
        self
    }
}

#[derive(Debug)]
struct ActionOutput {
    name: String,
    value: Box<dyn ToJson>,
}
#[derive(Debug)]
struct ActionInput {
    name: String,
    value: Box<dyn ToJson>,
}
impl PartialEq for ActionInput {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value.to_json() == other.value.to_json()
    }
}
impl Eq for ActionInput {}
impl PartialEq for ActionOutput {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value.to_json() == other.value.to_json()
    }
}
impl Eq for ActionOutput {}

trait ToJson: std::fmt::Debug {
    fn to_json(&self) -> String;
}
impl ToJson for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }
}
impl ToJson for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for &str {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_action_tree() {
        fn parent(parent: &mut ActionTree, x: i32) {
            fn child(parent: &mut ActionTree, name: &'static str, x: i32) {
                fn grandchild(
                    parent: &mut ActionTree,
                    id: i32,
                    name: &'static str,
                    x: i32,
                ) -> String {
                    let output = format!("grandchild: id={}, name={}, x={}", id, name, x);
                    let this = ActionTree::new("format")
                        .add_input("id", Box::new(id))
                        .add_input("name", Box::new(name))
                        .add_input("x", Box::new(x))
                        .add_output("output", Box::new(output.clone()));
                    parent.add_child(this);
                    output
                }
                let mut this = ActionTree::new("child")
                    .add_input("name", Box::new(name))
                    .add_input("x", Box::new(x));
                grandchild(&mut this, 0, name, x);
                parent.add_child(this);
            }
            let mut this = ActionTree::new("parent").add_input("x", Box::new(x));
            child(&mut this, "child", x);
            parent.add_child(this);
        }
        let mut root = ActionTree::root("TEST");
        parent(&mut root, 2);
        println!("{:#?}", root);
        assert_eq!(root, ActionTree::root("TEST"));
    }
}
