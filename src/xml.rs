use roxmltree::{Document, Error, Node};

pub fn parse(xml: &str) -> Result<Document, Error> {
    Document::parse(xml)
}

fn get_node<'a, 'input: 'a>(node: Node<'a, 'input>, path: &[&str]) -> Option<Node<'a, 'input>> {
    if path.is_empty() {
        return None;
    }

    let mut result = Some(node);

    for segment in path {
        result = result.and_then(|e| e.children().find(|e| e.tag_name().name() == *segment));
    }
    result
}

pub trait XmlExt {
    fn get_elem(&self, path: &[&str]) -> Option<Node<'_, '_>>;

    fn get_elem_text(&self, path: &[&str]) -> Option<&str>;
}

impl<'input> XmlExt for Document<'input> {
    fn get_elem(&self, path: &[&str]) -> Option<Node<'_, '_>> {
        let (root, path) = path.split_first()?;
        if self.root_element().tag_name().name() != *root {
            return None;
        }
        get_node(self.root_element(), path)
    }

    fn get_elem_text(&self, path: &[&str]) -> Option<&str> {
        let (root, path) = path.split_first()?;
        if self.root_element().tag_name().name() != *root {
            return None;
        }
        get_node(self.root_element(), path).and_then(|e| e.text())
    }
}

impl<'a, 'input: 'a> XmlExt for Node<'a, 'input> {
    fn get_elem(&self, path: &[&str]) -> Option<Node<'_, '_>> {
        get_node(*self, path)
    }

    fn get_elem_text(&self, path: &[&str]) -> Option<&str> {
        get_node(*self, path).and_then(|e| e.text())
    }
}
