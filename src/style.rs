use std::collections::HashMap;
use crate::dom::{ElementData, Node};
use crate::css::{SimpleSelector, Specificity, Stylesheet, Selector, Rule, Value};
use crate::dom::NodeType;

/// This article will cover what the CSS standard calls assigning property values,
/// or what I call the style module.
/// This module takes DOM nodes and CSS rules as input,
/// and matches them up to determine the value of each CSS property for any given node.
/// 本文将介绍 CSS 标准所说的分配属性值，或者我所说的样式模块。该模块将 DOM 节点和 CSS 规则作为输入，
/// 并将它们匹配以确定任何给定节点的每个 CSS 属性的值。

/// The output of the style module is something I call the style tree. Each node -
/// in this tree includes a pointer to a DOM node, plus its CSS property values:
/// 样式模块的输出就是我所说的样式树。这棵树中的每个节点都包含一个指向 DOM 节点的指针，以及它的 CSS 属性值：


/// Map from CSS property names to values.
/// 从 CSS 属性名称映射到值
type PropertyMap = HashMap<String, Value>;

/// A node with associated style data.
/// 具有关联样式数据的节点
pub struct StyledNode<'a> {
    pub node: &'a Node, // pointer to a DOM node 指向 DOM 节点的指针
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>
}

pub enum Display {
    Inline,
    Block,
    Node
}


/**
    What's with all the 'a stuff? Those are lifetimes,
    part of how Rust guarantees that pointers are memory-safe without requiring garbage collection.
    If you're not working in Rust you can ignore them; they aren't critical to the code's meaning.
    这些 'a 是什么？这些都是生存期，这是 Rust 如何保证指针是内存安全的，而不需要进行垃圾回收的部分原因。
    如果你不是在Rust的环境中工作，你可以忽略它们；它们对代码的意义并不重要。
 */


/// We could add new fields to the dom::Node struct instead of creating a new tree,
/// but I wanted to keep style code out of the earlier "lessons."This also gives me -
/// an opportunity to talk about the parallel trees that inhabit most rendering engines.
/// 我们可以将新字段添加到 dom::Node 结构而不是创建新树，但我想将样式代码排除在早期的“课程”之外。
/// 这也让我有机会谈论大多数渲染引擎中的并行树。


/// A browser engine module often takes one tree as input,
/// and produces a different but related tree as output. For example,
/// Gecko's layout code takes a DOM tree and produces a frame tree,
/// which is then used to build a view tree.
/// Blink and WebKit transform the DOM tree into a render tree.
/// Later stages in all these engines produce still more trees,
/// including layer trees and widget trees.
/// 浏览器引擎模块通常将一棵树作为输入，并生成一棵不同但相关的树作为输出。
/// 例如，Gecko 的布局代码采用 DOM 树并生成框架树，然后使用该框架树构建视图树。
/// Blink 和 WebKit 将 DOM 树转换为渲染树。所有这些引擎的后期阶段都会产生更多的树，包括层树和小部件树。

/// Selector matching
fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
    }
}

/// To test whether a simple selector matches an element, just look at each selector component,
/// and return false if the element doesn't have a matching class, ID, or tag name.
/// 要测试一个简单的选择器是否匹配一个元素，只需查看每个选择器组件，
/// 如果元素没有匹配的类、ID 或标签名称，则返回 false。
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {

    // Check type selector
    // 如果选择器的标签名跟元素的标签名不匹配，则 false；div != p
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check ID selector
    // id 不匹配
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    // 没有类名匹配
    let elem_classes = elem.classes();
    if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
        return false;
    }

    // We didn't find any non-matching selector components.
    // 都匹配
    return true;

    /*
        Rust note:
        此函数使用 any 方法，如果迭代器包含通过所提供测试的元素，则该方法返回 true。
        这与 Python（或 Haskell）中的 any 函数或 JavaScript 中的 some 方法相同。
     */
}

impl<'a> StyledNode<'a> {}

/// Next we need to traverse the DOM tree. For each element in the tree,
/// we will search the stylesheet for matching rules.
/// 接下来我们需要遍历 DOM 树。对于树中的每个元素，我们将在样式表中搜索匹配规则。

type MatchedRule<'a> = (Specificity, &'a Rule);

/// If `rule` matches `elem`, return a `MatchedRule`. Otherwise return `None`.
/// 如果 `rule` 匹配 `elem`，则返回 `MatchedRule`。否则返回 “None”
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    // Find the first (highest-specificity) matching selector.
    // 查找到第一个（最高优先级）匹配选择器。
    rule.selectors.iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

/// To find all the rules that match an element we call filter_map,
/// which does a linear scan through the style sheet,
/// checking every rule and throwing out ones that don't match.
/// A real browser engine would speed this up by storing the rules in multiple -
/// hash tables based on tag name, id, class, etc.
/// 为了找到与元素匹配的所有规则，我们调用 filter_map，它对样式表进行线性扫描，检查每个规则并丢弃不匹配的规则。
/// 真正的浏览器引擎会通过基于标签名称、id、类等将规则存储在多个哈希表中来加速这一过程。
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

/// Once we have the matching rules, we can find the specified values for the element.
/// We insert each rule's property values into a HashMap.We sort the matches by specificity,
/// so the more-specific rules are processed after the less-specific ones,
/// and can overwrite their values in the HashMap.
/// 一旦我们有了匹配规则，我们就可以找到元素的指定值。我们将每个规则的属性值插入到 HashMap 中。
/// 我们按优先级对匹配进行排序，因此更高优先级的规则在低优先级的规则之后处理，并且可以覆盖它们在 HashMap 中的值。

/// Apply styles to a single element, returning the specified values.
/// 将样式应用于单个元素，返回指定的值
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values: PropertyMap = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // Go through the rules from lowest to highest specificity.
    // 通过从最低到最高优先级的规则
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    values
}


/// Now we have everything we need to walk through the DOM tree and build the style tree.
/// Note that selector matching works only on elements,
/// so the specified values for a text node are just an empty map.
/// 现在我们拥有了遍历 DOM 树和构建样式树所需的一切。请注意，选择器匹配仅适用于元素，因此文本节点的指定值只是一个空映射。

/// Apply a stylesheet to an entire DOM tree, returning a StyledNode tree.
/// 将样式表应用到整个 DOM 树，返回一个 StyledNode 树
pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new()
        },
        children: root.children.iter()
            .map(|child| style_tree(child, stylesheet))
            .collect()
    }
}

impl<'a> StyledNode<'a> {
    /// Return the specified value of a property if it exists, otherwise `None`.
    /// 如果存在，则返回属性的指定值，否则返回 `None`
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }

    /// The value of the `display` property (defaults to inline).
    ///  `display` 属性的值（默认为内联）。
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::Node,
                _ => Display::Inline
            },
            _ => Display::Inline
        }
    }
}





