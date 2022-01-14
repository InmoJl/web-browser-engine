use std::collections::HashMap;
use std::iter::Peekable;
use std::str::CharIndices;
use proc_macro::bridge::PanicMessage::String;
use crate::dom;

pub struct Parser {
    pos: usize,
    input: String
}

impl Parser {

    // Read the current character without consuming it.
    // 读取下一个字符
    // 读取当前字符而不消耗他
    fn next_char(&self) -> char {
        // 获取当前位置到结束的字符串 slice
        // chars 获取字符迭代器
        // next 获取迭代器的下一个值，第一次 next 获取的是第一个值
        self.input[self.pos..].chars().next().unwrap()
    }

    // Do the next characters start with the given string
    // 字符是否以 `s` 开头
    // 下一个字符是否以给定字符串开头
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // Return ture if all input is consumed
    // 是否到结尾
    // 如果所有字符被消耗了，则返回 true
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // Return thr current character, and advance `self.pos` to the next character
    // 消耗一个字符
    // 返回当前的字符，并且 pos 指向下一个字符位置
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices().peekable();
        // 获取下一个的值
        let (_, cur_char) = iter.next().unwrap();

        // 忽略注释
        if cur_char == '<' && self.starts_with("<!--") {
            self.parse_comment(iter);
        }

        // 获取下一个的值的位置
        // 如果 `unwrap` 后的值是 `None`，则返回一个默认值 `(1, ' ')`
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        return cur_char;
    }

    // Parse comment, like `<!-- contents -->`
    fn parse_comment(&mut self, mut iter: Peekable<CharIndices>) {
        // 消费掉 3 个字符（<!--）
        // iter.next();iter.next();iter.next();
        iter.skip(3);
        let mut closing_str = String::new();
        loop {

            // 碰到文档结束，说明没有闭合标签
            assert_eq!(self.eof(), false);

            // 已匹配结束
            if closing_str == "-->" {
                break;
            }

            // 更新 pos 位置
            let (next_pos, cur_char) = iter.next().unwrap_or((1, ' '));
            self.pos = next_pos;

            // 碰到结束标签
            if cur_char == '-' {
                closing_str.push(cur_char);
            }

        }
    }

    // Consume characters until `test` returns false
    // 条件循环消耗
    // 通常我们需要使用消耗一串连续的字符，consume_while 方法消耗满足给定条件的字符
    // 只要用过 `test` 函数，就会进行消耗
    // 并将消耗掉的字符返回
    fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {

        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }

        return result;
    }

    // Consume and discard zero or more whitespace characters.
    // 使用并丢弃零个或多个空白字符
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // Parse a tag or attribute name.
    // 解析标签或者属性名，匹配到不是 `a-zA-Z0-9` 的就直接返回
    // 即标签名或者属性名只支持 `a-zA-Z0-9`
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' => true,
            _ => false
        })
    }

    // Parse a single node.
    // 解析单个节点
    // 根据第一个字符判断是元素还是文本节点，除了包含 `<` 以外的任何字符
    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text()
        }
    }

    // Parse a text node.
    // 匹配之到下一个元素节点，判断条件为 '<'
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    // Parse a single element, including its open tag, contents, and closing tag.
    // 解析的单个元素，它包含开标签，内容和闭合标签
    fn parse_element(&mut self) -> dom::Node {

        // Opening tag.
        // 匹配开标签、标签名与属性名。在这前后进行标签字符断言
        assert_eq!(self.consume_char(), '<');
        let tag_name = self.parse_tag_name();
        // 在开标签中匹配属性
        let attrs = self.parse_attributes();
        assert_eq!(self.consume_char(), '>');

        // Contents.
        // 元素内容
        let children = self.parse_nodes();

        // Closing tag.
        // 解析闭合标签
        assert_eq!(self.consume_char(), '<');
        assert_eq!(self.consume_char(), '/');
        assert_eq!(self.parse_tag_name(), tag_name);
        assert_eq!(self.consume_char(), '>');

        dom::element(tag_name, attrs, children)
    }

    // Parse a single name="value" pair
    // 解析单个属性对
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        // 属性名后面跟着 `=`
        assert_eq!(self.consume_char(), '=');
        let value = self.parse_attr_value();
        return (name, value);
    }

    // Parse a quoted value.
    // 解析引号包含的属性值
    fn parse_attr_value(&mut self) -> String {
        // 消费一个字符
        let open_quote = self.consume_char();
        // 判断是否是 `"` or `'`
        assert!(open_quote == '"' || open_quote == '\'');
        // 成对匹配，如果使用 `"` 包含属性值，则闭合必须也是 `"`
        let value = self.consume_while(|c| c != open_quote);
        assert_eq!(self.consume_char(), open_quote);
        return value;
    }

    // Parse a list of name="value" pairs, separated by whitespace
    // 解析以空格分割的键值对列表：a="b" c="d"
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();

        // 进这里意味着是在开标签内 <here>
        // 读取匹配字符，知道遇上 > 结束符
        loop {

            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }

            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }

        return attributes;
    }

    // Parse a sequence of sibling nodes.
    // 解析一系列兄弟节点
    // 为了解析子节点，我们在循环中递归调用 parse_node 直到到达闭合标记
    // 这个函数返回一个 Vec，它是 Rust 的可增长数组的名称
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }

        return nodes;
    }

    // Parse an HTML document and return the root element.
    // 解析 HTML 文档并返回根元素
    // 将所有这些放在一起，将整个 HTML 文档解析为 DOM 树
    // 如果文档没有明确包含根节点，此函数将为文档创建一个根节点；这类似于真正的 HTML 解析器所做的
    pub fn parse(source: String) -> dom::Node {
        let mut nodes = Parser { pos: 0, input: source }.parse_nodes();

        // If the document contains a root element, just return it. Otherwise, create one.
        // 如果文档包含根元素，则返回它。否则，创建一个
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            dom::element("html".to_string(), HashMap::new(), nodes)
        }
    }
}


