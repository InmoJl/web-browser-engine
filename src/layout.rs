use crate::css::Stylesheet;
use crate::layout::BoxType::{BlockNode, InlineNode};
use crate::style::{ StyledNode, Display };
use crate::style::Display::Block;

/// Layout is all about boxes. A box is a rectangular section of a web page. It has a width,
/// a height, and a position on the page.
/// This rectangle is called the content area because it's where the box's content is drawn.
/// The content may be text, image, video, or other boxes.
/// 布局就是方框。方框是网页的一个矩形部分。它具有页面上的宽度、高度和位置。
/// 这个矩形称为内容区域，因为它是框的内容绘制的地方。内容可以是文本、图像、视频或其他框。

/// A box may also have padding, borders, and margins surrounding its content area.
/// The CSS spec has a diagram showing how all these layers fit together.
/// 框还可以在其内容区域周围有内边距、边框和边距。CSS规范中有一个图表显示所有这些层是如何组合在一起的。



/// CSS box model. All sizes are in px.
/// CSS 盒子模型。所有尺寸均以 px 为单位
struct Dimensions {
    /// 内容区域相对于文档原点的位置：
    content: Rect,

    /// Surrounding edges:
    /// 周围边距
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes
}

struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32
}



/// The CSS display property determines which type of box an element generates.
/// CSS defines several box types, each with its own layout rules.
/// I'm only going to talk about two of them: block and inline.
/// CSS display 属性确定元素生成哪种类型的框。 CSS 定义了几种盒子类型，每一种都有自己的布局规则。
/// 我只讨论其中的两个：块和内联。



/// Each box must contain only block children, or only inline children.
/// When an DOM element contains a mix of block and inline children,
/// the layout engine inserts anonymous boxes to separate the two types.
/// (These boxes are "anonymous" because they aren't associated with nodes in the DOM tree.)
/// 每个框必须仅包含块子级，或仅包含内联子级。当 DOM 元素包含块和内联子元素的混合时，
/// 布局引擎会插入匿名框来分隔这两种类型。 （这些框是“匿名的”，因为它们与 DOM 树中的节点无关。）


/// The layout tree is a collection of boxes. A box has dimensions, and it may contain child boxes.
/// 布局树是框的集合。一个盒子有尺寸，它可能包含子盒子。
struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>
}

/// A box can be a block node, an inline node, or an anonymous block box.
/// (This will need to change when I implement text layout,
/// because line wrapping can cause a single inline node to split into multiple boxes.
/// But it will do for now.)
/// 盒子可以是块节点、内联节点或匿名块盒子。
/// （当我实现文本布局时，这需要更改，因为换行会导致单个内联节点拆分为多个框。但现在可以。）
enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock
}



/// To build the layout tree, we need to look at the display property for each DOM node.
/// I added some code to the style module to get the display value for a node.
/// If there's no specified value it returns the initial value, 'inline'.
/// 要构建布局树，我们需要查看每个 DOM 节点的显示属性。
/// 我在样式模块中添加了一些代码来获取节点的显示值。如果没有指定值，则返回初始值 'inline'


/// Now we can walk through the style tree, build a LayoutBox for each node,
/// and then insert boxes for the node's children.
/// If a node's display property is set to 'none' then it is not included in the layout tree.
/// 现在我们可以遍历样式树，为每个节点构建一个 LayoutBox，然后为节点的子节点插入框。
/// 如果节点的显示属性设置为“无”，则它不包含在布局树中。



/// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
/// 构建 LayoutBoxes 树，但不执行任何布局计算
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BlockNode(style_node),
        Display::Inline => InlineNode(style_node),
        Display::Node => panic!("Root node has display: none.")
    });

    // Create the descendant boxes.
    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(chlld)),
            Display::Node => {}
        }
    }

    root
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: Vec::new(),
        }
    }

    // Where a new inline child should go.
    // 一个新的内联子元素应该去哪里
    fn get_inline_container(&mut self) -> &mut LayoutBox {
        match self.box_type {
            InlineNode(_) | BoxType::AnonymousBlock => self,
            BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                // 如果我们刚刚生成了一个匿名块框，请继续使用它
                // 否则，创建一个新的
                match self.children.last() {
                    Some(&LayoutBox {box_type: BoxType::AnonymousBlock,..}) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}



