use std::default::Default;
use crate::style::{ StyledNode, Display };
use crate::css::Unit::Px;
use crate::css::Value::{Keyword, Length};

pub use self::BoxType::{AnonymousBlock, InlineNode, BlockNode};

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
#[derive(Clone, Copy, Default, Debug)]
pub struct Dimensions {
    /// 内容区域相对于文档原点的位置：
    pub content: Rect,

    /// Surrounding edges:
    /// 周围边距
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

#[derive(Clone, Copy, Default, Debug)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32
}



/// The CSS display property determines which type of box an element generates.
/// CSS defines several box types, each with its own layout rules.
/// I'm only going to talk about two of them: block and inline.
/// CSS display 属性确定元素生成哪种类型的框。 CSS 定义了几种盒子类型，每一种都有自己的布局规则。
/// 我只讨论其中的两个：block 和 inline。



/// Each box must contain only block children, or only inline children.
/// When an DOM element contains a mix of block and inline children,
/// the layout engine inserts anonymous boxes to separate the two types.
/// (These boxes are "anonymous" because they aren't associated with nodes in the DOM tree.)
/// 每个框必须仅包含块子级，或仅包含内联子级。当 DOM 元素包含块和内联子元素的混合时，
/// 布局引擎会插入匿名框来分隔这两种类型。 （这些框是“匿名的”，因为它们与 DOM 树中的节点无关。）



/// The layout tree is a collection of boxes. A box has dimensions, and it may contain child boxes.
/// 布局树是框的集合。一个盒子有尺寸，它可能包含子盒子。
#[derive(Debug)]
pub struct LayoutBox<'a> {
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
#[derive(Debug)]
pub enum BoxType<'a> {
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

/// Transform a style tree into a layout tree.
/// 将样式树转换为布局树
pub fn layout_tree<'a>(node: &'a StyledNode<'a>, mut containing_block: Dimensions) -> LayoutBox<'a> {
    // The layout algorithm expects the container height to start at 0.
    // TODO: Save the initial containing block height, for calculating percent heights.
    // 布局算法期望容器高度从 0 开始。
    // TODO: 保存初始包含块高度，用于计算百分比高度
    containing_block.content.height = 0.0;

    let mut root_box = build_layout_tree(node);
    root_box.layout(containing_block);

    root_box
}

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
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            Display::Node => {}
        }
    }

    root
}

impl<'a> LayoutBox<'a> {

    /// The entry point to this code is the layout function,
    /// which takes a takes a LayoutBox and calculates its dimensions.
    /// We'll break this function into three cases, and implement only one of them for now:
    /// 这段代码的入口点是 layout 函数，它接受一个 LayoutBox 并计算其尺寸。我们将把这个函数分成三种情况，现在只实现其中一种：
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => {},
            BoxType::AnonymousBlock => {}
        }
    }

    /// A block's layout depends on the dimensions of its containing block.
    /// For block boxes in normal flow, this is just the box's parent.
    /// For the root element, it's the size of the browser window (or "viewport").
    /// 块的布局取决于其包含块的尺寸。对于正常流程中的块框，这只是框的父级。对于根元素，它是浏览器窗口（或“视口”）的大小

    /// You may remember from the previous article that a block's width depends on its parent,
    /// while its height depends on its children.
    /// This means that our code needs to traverse the tree top-down while calculating widths,
    /// so it can lay out the children after their parent's width is known,
    /// and traverse bottom-up to calculate heights,
    /// so that a parent's height is calculated after its children's.
    /// 你可能还记得之前的文章，块的宽度取决于它的父级，而它的高度取决于它的子级。
    /// 这意味着我们的代码在计算宽度时需要自顶向下遍历树，所以它可以在知道父级宽度后对子级进行布局，
    /// 并自底向上遍历计算高度，从而在计算父级高度后计算其子级
    fn layout_block(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width, so we need to calculate
        // 子宽度可以依赖于父宽度，所以我们需要计算
        // this box's width before laying out its children.
        // 这个盒子在布局它的孩子之前的宽度
        self.calculate_block_width(containing_block);

        // Determine where the box is located within its container.
        // 确定盒子在其容器内的位置
        self.calculate_block_position(containing_block);

        // Recursively lay out the children of this box.
        // 递归地布置这个盒子的子元素
        self.layout_block_children();

        // Parent height can depend on child height, so `calculate_height`
        // must be called *after* the children are laid out.
        // 父级高度可以依赖于子级高度，所以 `calculate_height` 必须在子级布局后调用
        self.calculate_block_height();

        // This function performs a single traversal of the layout tree,
        // doing width calculations on the way down and height calculations on the way back up.
        // A real layout engine might perform several tree traversals, some top-down and some bottom-up.
        // 此函数执行布局树的单次遍历，向下计算宽度，向上计算高度。一个真正的布局引擎可能会执行几个树遍历，一些自顶向下和一些自底向上。

        // And that concludes the block layout algorithm.
        // You can now call layout() on a styled HTML document,
        // and it will spit out a bunch of rectangles with widths, heights, margins, etc. Cool, right?
        // 块布局算法到此结束。您现在可以在样式化的 HTML 文档上调用 layout()，它会吐出一堆具有宽度、高度、边距等的矩形。很酷，对吗？
    }

    /// Calculate the width of a block-level non-replaced element in normal flow.
    /// Sets the horizontal margin/padding/border dimensions, and the `width`.
    /// 计算正常流中块级非替换元素的宽度
    /// 设置水平方向的 margin/padding/border 的尺寸, 和 `width`.
    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // 'width' has initial value 'auto'
        let auto = Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0
        let zero = Length(0.0, Px);


        // This uses a helper function called lookup, which just tries a series of values in sequence.
        // If the first property isn't set, it tries the second one. If that's not set either,
        // it returns the given default value.
        // This provides an incomplete (but simple) implementation of shorthand properties and initial values.
        // 这使用了一个名为lookup 的辅助函数，它只是按顺序尝试一系列值。如果第一个属性没有设置，它会尝试第二个。
        // 如果也没有设置，则返回给定的默认值。这提供了速记属性和初始值的不完整（但简单）实现。


        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);


        // Since a child can't change its parent's width,
        // it needs to make sure its own width fits the parent's.
        // The CSS spec expresses this as a set of constraints and an algorithm for solving them.
        // The following code implements that algorithm.
        // 由于子元素不能改变其父元素的宽度，它需要确保自己的宽度适合父元素的。
        // CSS 规范将其表示为一组约束和解决它们的算法。以下代码实现了该算法。

        // First we add up the margin, padding, border, and content widths.
        // The to_px helper method converts lengths to their numerical values.
        // If a property is set to 'auto', it returns 0 so it doesn't affect the sum.
        // 首先，我们将边距、内边距、边框和内容宽度相加。 to_px 辅助方法将长度转换为它们的数值。
        // 如果属性设置为“自动”，则返回 0，因此不会影响总和。

        let total = sum(
            [
                &margin_left, &margin_right,
                &border_left, &border_right,
                &padding_left, &padding_right,
                &width
            ].iter().map(|v| v.to_px())
        );

        // This is the minimum horizontal space needed for the box.
        // If this isn't equal to the container width, we'll need to adjust something to make it equal.
        // 这是盒子所需的最小水平空间。如果这不等于容器宽度，我们需要调整一些东西以使其相等。

        // If the width or margins are set to 'auto', they can expand or contract to fit the available space.
        // Following the spec, we first check if the box is too big. If so, we set any expandable margins to zero.
        // 如果宽度或边距设置为“auto”，它们可以扩展或收缩以适应可用空间。按照规范，我们首先检查盒子是否太大。
        // 如果是这样，我们将任何可扩展边距设置为零
        // If width is not auto and the total is wider than the container, treat auto margins as 0.
        // 如果 width 不是 auto 并且比容器宽，则将 auto 边距设为 0。
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Px);
            }

            if margin_right == auto {
                margin_right = Length(0.0, Px);
            }
        }


        // If the box is too large for its container, it overflows the container.
        // If it's too small, it will underflow, leaving extra space.
        // We'll calculate the underflow—the amount of extra space left in the container.
        // (If this number is negative, it is actually an overflow.)
        // 如果盒子对于它的容器来说太大了，它就会溢出容器。如果它太小，它会下溢，留下额外的空间。
        // 我们将计算下溢——容器中剩余的额外空间量。（如果这个数字是负数，它实际上是一个溢出。）

        // Adjust used values so that the above sum equals `containing_block.width`.
        // Each arm of the `match` should increase the total width by exactly `underflow`,
        // and afterward all values should be absolute lengths in px.
        // 调整使用的值，使上述总和等于 `containing_block.width`
        // `match` 的每个分支都应该增加 `underflow` 的总宽度
        // 之后的值都应该是 px 的绝对长度
        let underflow = containing_block.content.width - total;

        // We now follow the spec's algorithm for eliminating any overflow or underflow by adjusting the expandable dimensions.
        // If there are no 'auto' dimensions, we adjust the right margin.
        // (Yes, this means the margin may be negative in the case of an overflow!)
        // 我们现在遵循规范的算法，通过调整可扩展尺寸来消除任何溢出或下溢。
        // 如果没有“auto”尺寸，我们会调整右边距。（是的，这意味着在溢出的情况下边距可能为负！）
        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            // 如果值过度约束，则计算 margin_right
            (false, false, false) => {
                margin_right = Length(margin_right.to_px() + underflow, Px);
            }

            // If exactly one size is auto, its used value follows from the equality.
            // 如果恰好有一种尺寸是自动的，则其使用的值遵循等式。
            (false, true, false) => { margin_left  = Length(underflow, Px); }
            (false, false, true) => { margin_right = Length(underflow, Px); }

            // If width is set to auto, any other auto values become 0.
            // 如果宽度设置为自动，任何其他自动值都变为 0
            (true, _, _) => {
                if margin_left == auto { margin_left = Length(0.0, Px); }
                if margin_right == auto { margin_right = Length(0.0, Px); }

                if underflow >= 0.0 {
                    width = Length(underflow, Px);
                } else {
                    width = Length(0.0, Px);
                    margin_right = Length(margin_right.to_px() + underflow, Px);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            // 如果 margin-left 和 margin-right 都是自动的，它们使用的值是相等的
            (false, true, true) => {
                margin_left = Length(underflow / 2.0, Px);
                margin_right = Length(underflow / 2.0, Px);
            }
        }


        // 保存计算值

        let d = &mut self.dimensions;
        d.content.width = width.to_px();

        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();

        // At this point, the constraints are met and any 'auto' values have been converted to lengths.
        // The results are the the used values for the horizontal box dimensions, which we will store in the layout tree.
        // 此时，满足约束并且任何“auto”值都已转换为长度。结果是水平框尺寸的使用值，我们将存储在布局树中
    }

    /// Finish calculating the block's edge sizes, and position it within its containing block.
    /// Sets the vertical margin/padding/border dimensions, and the `x`, `y` values.
    /// 完成计算 block 的边缘大小，并将其定位在其包含的块中
    /// 设置垂直 margin/padding/border 尺寸，以及 x,y 值
    /// The next step is simpler. This function looks up the remanining margin/padding/border styles,
    /// and uses these along with the containing block dimensions to determine this block's position on the page.
    /// 此函数查找剩余边距/填充/边框样式，并使用这些与包含块尺寸一起确定此块在页面上的位置。
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Px);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        // 将框定位在容器中所有先前框的下方
        d.content.y = containing_block.content.height + containing_block.content.y +
                        d.margin.top + d.border.top + d.padding.top;

        // Take a close look at that last statement, which sets the y position.
        // This is what gives block layout its distinctive vertical stacking behavior.
        // For this to work, we'll need to make sure the parent's `content.height` is updated after laying out each child.
        // 仔细看看最后一条语句，它设置了 y 位置。这就是让块布局具有独特的垂直堆叠行为的原因。
        // 为此，我们需要确保在布置每个孩子之后更新父母的 content.height
    }

    /// Here's the code that recursively lays out the box's contents.
    /// As it loops through the child boxes, it keeps track of the total content height.
    /// This is used by the positioning code (above) to find the vertical position of the next child.
    /// 这是递归布置盒子内容的代码。当它遍历子框时，它会跟踪总内容高度。定位代码（上图）使用它来查找下一个孩子的垂直位置
    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            // Track the height so each child is laid out below the previous content.
            // 跟踪高度，以便将每个子项放置在前一个内容的下方
            d.content.height = d.content.height + child.dimensions.margin_box().height;
        }
    }

    /// By default, the box's height is equal to the height of its contents.
    /// But if the 'height' property is set to an explicit length, we'll use that instead:
    /// 默认情况下，盒子的高度等于其内容的高度。但是如果 'height' 属性设置为显式长度，我们将使用它来代替：
    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // 如果高度设置为显式长度，则使用该确切长度
        // Otherwise, just keep the value set by `layout_block_children`.
        // 否则，只需保留 `layout_block_children` 设置的值
        if let Some(Length(h, Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }

    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node")
        }
    }

    // Where a new inline child should go.
    // 一个新的内联子元素应该去哪里
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            InlineNode(_) | BoxType::AnonymousBlock => self,
            BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                // 如果我们刚刚生成了一个匿名块框，请继续使用它
                // 否则，创建一个新的
                match self.children.last() {
                    Some(&LayoutBox {box_type: BoxType::AnonymousBlock, ..}) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

impl Rect {
    pub fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom
        }
    }
}


// The total vertical space taken up by each child is the height of its margin box
// 每个孩子占据的总垂直空间是其边距框的高度

impl Dimensions {
    /// The area covered by the content area plus its padding.
    /// 内容区域加上它的 padding 所覆盖的区域
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    /// The area covered by the content area plus padding and borders.
    /// 内容区域加上 padding 和 border 所覆盖的区域
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    /// The area covered by the content area plus padding, borders, and margin.
    /// 内容区域加上 padding、border 和 margin 所覆盖的区域
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

// For simplicity, this does not implement margin collapsing.
// A real layout engine would allow the bottom margin of one box to overlap the top margin of the next box,
// rather than placing each margin box completely below the previous one.
// 为简单起见，这并没有实现边距折叠。真正的布局引擎将允许一个框的下边距与下一个框的上边距重叠，而不是将每个边距框完全放在前一个框的下方

fn sum<I>(iter: I) -> f32 where I: Iterator<Item=f32> {
    iter.fold(0., |a, b| a + b)
}
