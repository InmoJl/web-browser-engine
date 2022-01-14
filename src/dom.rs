use std::collections::{HashMap, HashSet};

pub struct ElementData {
    tag_name: String,
    attributes: AttrMap
}

pub type AttrMap = HashMap<String, String>;

pub enum NodeType {
    Text(String),
    Element(ElementData)
}

pub struct Node {
    children: Vec<Node>,
    node_type: NodeType
}

// 生成一个文本节点
pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data)
    }
}

// 生成一个元素节点
pub fn element(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs
        })
    }
}

// Element methods

impl ElementData {

    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(class_list) => class_list.split(' ').collect(),
            None => HashSet::new()
        }
    }
}
