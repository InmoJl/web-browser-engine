use core::panicking::panic;

// A CSS stylesheet is a series of rules.
// CSS 样式表是一系列规则
pub struct Stylesheet {
    pub rules: Vec<Rule>
}

// A rule includes one or more selectors separated by commas
// followed by a series of declarations enclosed in braces.
// Rule 包括一个或多个用逗号分隔的选择器，后跟一系列用大括号括起来的声明。
pub struct Rule {
    pub selectors: Vec<Selector>, // .box
    pub declarations: Vec<Declaration> // 属性声明：[{name:value}]
}

// A selector can be a simple selector, or it can be a chain of selectors joined by combinators.
// 选择器可以是简单的选择器，也可以是组合连接起来的选择器链
// a simple selector can include a tag name
// an ID prefixed by '#', any number of class names prefixed by '.'
// or some combination of the above.
// 一个简单的选择器可以包含一个标签名称、一个以“#”为前缀的 ID、任意数量的以“.”为前缀的类名，或者上述的某种组合
// If the tag name is empty or '*' then it is a "universal selector" that can match any tag.
// 如果标签名称为空或'*'，那么它是一个可以匹配任何标签的“通用选择器”

#[deriving(Show)]
pub enum Selector {
    Simple(SimpleSelector)
}

pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>
}

/*
    Specificity is one of the ways a rendering engine decides which style overrides the other in a conflict.
    If a stylesheet contains two rules that match an element,
    the rule with the matching selector of higher specificity can override values from the one with lower specificity.
    当样式出现冲突，优先级是渲染引擎决定如何覆盖另外一种样式的方式之一
    如果样式表包含两个匹配元素的规则，则具有较高优先级的匹配选择器的规则可以覆盖来自较低优先级的规则的值
*/

/*
    The specificity of a selector is based on its components.
    An ID selector is more specific than a class selector,
    which is more specific than a tag selector.
    Within each of these "levels," more selectors beats fewer.
    选择器的优先级基于其组件。 ID 选择器比类选择器更具体，类选择器比标签选择器更具体。
    在这些 "层级" 中，选择器越多优先级更高
 */

/// 优先级
pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();

        (a, b, c)
    }
}

/// If we supported chained selectors,
/// we could calculate the specificity of a chain just by adding up the specificities of its parts.
/// 如果我们支持链式选择器，我们可以通过将其部分的优先级相加来计算链的优先级


// A declaration is just a name/value pair, separated by a colon and ending with a semicolon.
// For example, "margin: auto;" is a declaration. For example, "margin: auto;" is a declaration.
// 一个 "Declaration" 只是一个 name/value 的键值对，用冒号分隔并以分号结尾
// 例如："margin: auto;" 是一个 "Declaration"
struct Declaration {
    pub name: String,
    pub value: String
}

// supports only a handful of CSS's many value types.
// 仅支持少数 CSS 的值类型
enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values here
}

enum Unit {
    Px,
    // insert more units here
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

// All other CSS syntax is unsupported, including @-rules, comments, and any selectors/values/units not mentioned above.
// 不支持所有其他 CSS 语法，包括 @-rules、注释和上面未提及的任何 selectors/values/units



/**
    CSS has a straightforward grammar, making it easier to parse correctly than its quirky cousin HTML.
    When a standards-compliant CSS parser encounters a parse error,
    it discards the unrecognized part of the stylesheet but still processes the remaining portions.
    This is useful because it allows stylesheets to include new syntax but still produce well-defined output in older browsers.
    CSS 有一个简单的语法，比它古怪的表亲 HTML 更容易正确解析。当符合标准的 CSS 解析器遇到解析错误时，
    它会丢弃样式表中无法识别的部分，但仍会处理剩余部分。
    这很有用，因为它允许样式表包含新语法，但仍然在旧浏览器中产生定义明确的输出。
 */

/**
    A very simplistic (and totally not standards-compliant) parser,
    built the same way as the HTML parser from Part 2.
    Rather than go through the whole thing line-by-line again, I'll just paste in a few snippets.
    For example, here is the code for parsing a single selector:
    一个非常简单（并且完全不符合标准）的解析器，其构建方式与第 2 部分中的 HTML 解析器相同。
    我不会再次逐行查看整个内容，而是粘贴几个片段.
    例如，下面是解析单个选择器的代码
 */


struct Parser {
    pos: usize,
    input: String
}

impl Parser {

    /// The selectors for each rule are stored in a sorted vector,
    /// most-specific first. This will be important in matching,
    /// which I'll cover in the next article.
    /// 每个规则的选择器存储在一个排序的 vector 中，优先级最高的优先。这在匹配中很重要，我将在下一篇文章中介绍。


    /// Parse a rule set: `<selectors> { <declarations> }`.
    /// 解析规则集
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations()
        }
    }

    /// Parse a comma-separated list of selectors.
    /// 解析以逗号分隔的选择器列表
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break, // start of declarations
                c => panic!("Unexpected character {} in selector list", c)
            }
        }

        /// Return selectors with highest specificity first, for use in matching.
        /// 首先返回具有最高优先级的选择器，用于匹配
        /// 比较选择器
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));

        return selectors;
    }


    // Parse one simple selector, e.g.: `type#id.class1.class2.class3`
    // 解析一个简单的选择器，例如：`type#id.class1.class2.class3`
    // Note the lack of error checking.
    // Some malformed input like ### or *foo* will parse successfully and produce weird results.
    // A real CSS parser would discard these invalid selectors.
    // 请注意缺少错误检查。一些格式错误的输入，如 ### 或 *foo* 将成功解析并产生奇怪的结果。
    // 真正的 CSS 解析器会丢弃这些无效的选择器。
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector { tag_name: None, id: None, class: Vec::new() };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break
            }
        }

        selector
    }

    /// Parse a property name or keyword
    /// 解析属性名称或关键字
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    // 根据条件，消耗一系列字符
    fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // 结束
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // 获取下一个字符，不消耗
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // 消费一个字符
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_post, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_post;
        cur_char
    }

    /// Consume and discard zero or more whitespace characters.
    /// 消费掉空格字符
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

}

// 判断输入的字符是否是允许使用的字符
fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true, // TODO: Include U+00A0 and higher.
        _ => false,
    }
}

