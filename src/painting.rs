// In this article, I will add very basic painting code.
// This code takes the tree of boxes from the layout module and turns them into an array of pixels.
// This process is also known as "rasterization."
// 在本文中，我将添加非常基本的绘画代码。此代码从布局模块中获取框树并将它们转换为像素数组。此过程也称为“光栅化”。

// Before painting, we will walk through the layout tree and build a display list.
// This is a list of graphics operations like "draw a circle" or "draw a string of text." Or in our case, just "draw a rectangle."
// 在绘制之前，我们将遍历布局树并构建一个显示列表。这是一个图形操作列表，例如“画一个圆”或“画一串文本”。
// 或者在我们的例子中，只是“画一个矩形”。

// Why put commands into a display list, rather than execute them immediately?
// The display list is useful for a several reasons.
// You can search it for items that will be completely covered up by later operations,
// and remove them to eliminate wasted painting.
// You can modify and re-use the display list in cases where you know only certain items have changed.
// And you can use the same display list to generate different types of output: for example,
// pixels for displaying on a screen, or vector graphics for sending to a printer.
// 为什么将命令放入显示列表，而不是立即执行？显示列表很有用有几个原因。
// 您可以在其中搜索将被以后的操作完全覆盖的项目，并将其删除以消除浪费的绘画。
// 如果您知道仅某些项目已更改，则可以修改和重新使用显示列表。您可以使用相同的显示列表生成不同类型的输出：
// 例如，用于在屏幕上显示的像素，或用于发送到打印机的矢量图形

// Robinson's display list is a vector of DisplayCommands.
// For now there is only one type of DisplayCommand, a solid-color rectangle:
// Robinson 的显示列表是一个 DisplayCommands 的向量。目前只有一种 DisplayCommand，即纯色矩形:

use crate::css::Color;
use crate::css::Value;
use crate::layout::{LayoutBox, Rect, BlockNode, InlineNode, AnonymousBlock};

#[derive(Debug)]
pub enum DisplayCommand {
    SolidColor(Color, Rect)
    // insert more commands here
}

pub type DisplayList = Vec<DisplayCommand>;

// To build the display list, we walk through the layout tree and generate a series of commands for each box.
// First we draw the box's background, then we draw its borders and content on top of the background.
// 为了构建显示列表，我们遍历布局树并为每个框生成一系列命令。首先我们绘制盒子的背景，然后我们在背景之上绘制它的边框和内容

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

// By default, HTML elements are stacked in the order they appear: If two elements overlap,
// the later one is drawn on top of the earlier one. This is reflected in our display list,
// which will draw the elements in the same order they appear in the DOM tree.
// If this code supported the z-index property,
// then individual elements would be able to override this stacking order,
// and we'd need to sort the display list accordingly.
// 默认情况下，HTML 元素按照它们出现的顺序堆叠：如果两个元素重叠，则后面的元素将绘制在前面的元素之上。
// 这反映在我们的显示列表中，它将按照元素在 DOM 树中出现的顺序绘制元素。
// 如果此代码支持 z-index 属性，那么单个元素将能够覆盖此堆叠顺序，我们需要相应地对显示列表进行排序

// The background is easy. It's just a solid rectangle.If no background color is specified,
// then the background is transparent and we don't need to generate a display command.
// 背景很简单。它只是一个实心矩形。如果没有指定背景颜色，那么背景是透明的，我们不需要生成显示命令。

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(color) = get_color(layout_box, "background") {
        list.push(DisplayCommand::SolidColor(color, layout_box.dimensions.border_box()));
    }
}

/// Return the specified color for CSS property `name`, or None if no color was specified.
/// 返回 CSS 属性 `name` 的指定颜色，如果没有指定颜色，则返回 None
fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BlockNode(style) | InlineNode(style) => match style.value(name) {
            Some(Value::ColorValue(color)) => Some(color),
            _ => None
        },
        AnonymousBlock => None
    }
}

// The borders are similar, but instead of a single rectangle we draw four—one for each edge of the box.
// 边框是相似的，但我们绘制的不是一个矩形，而是四个矩形——一个用于框的每个边缘

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    // 获取边框颜色
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return
    };

    let d = &layout_box.dimensions;
    let border_box = d.border_box();

    // Left border
    list.push(DisplayCommand::SolidColor(color, Rect {
        x: border_box.x,
        y: border_box.y,
        width: d.border.left,
        height: border_box.height
    }));

    // Right border
    list.push(DisplayCommand::SolidColor(color, Rect {
        x: border_box.x + border_box.width - d.border.right,
        y: border_box.y,
        width: d.border.right,
        height: border_box.height
    }));

    // Top border
    list.push(DisplayCommand::SolidColor(color, Rect {
        x: border_box.x,
        y: border_box.y,
        width: border_box.width,
        height: d.border.top
    }));

    // Bottom border
    list.push(DisplayCommand::SolidColor(color, Rect {
        x: border_box.x,
        y: border_box.y + border_box.height - d.border.bottom,
        width: border_box.width,
        height: d.border.bottom
    }));

}

// Next the rendering function will draw each of the box's children,
// until the entire layout tree has been translated into display commands.
// 接下来，渲染函数将绘制每个盒子的孩子，直到整个布局树被翻译成显示命令。

// Now that we've built the display list,
// we need to turn it into pixels by executing each DisplayCommand. We'll store the pixels in a Canvas
// 现在我们已经构建了显示列表，我们需要通过执行每个 DisplayCommand 将其转换为像素。我们将像素存储在 Canvas 中
#[derive(Debug)]
pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize
}

impl Canvas {
    ///  Create a blank canvas
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color { r: 255, g: 255, b: 255, a: 255 };
        Canvas {
            width,
            height,
            pixels: vec![white; width * height]
        }
    }

    /// To paint a rectangle on the canvas, we just loop through its rows and columns,
    /// using a helper method to make sure we don't go outside the bounds of our canvas.
    /// 要在画布上绘制一个矩形，我们只需遍历它的行和列，使用辅助方法来确保我们不会超出画布的边界
    fn paint_item(&mut self, item: &DisplayCommand) {
        match *item {
            DisplayCommand::SolidColor(color, rect) => {
                let x0 = rect.x.clamp(0.0, self.width as f32) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;

                for y in y0 .. y1 {
                    for x in x0 .. x1 {
                        // TODO: alpha compositing with existing pixel
                        self.pixels[y * self.width + x] = color;
                    }
                }
            }
        }

        // Note that this code only works with opaque colors.
        // If we added transparency (by reading the opacity property,
        // or adding support for rgba() values in the CSS parser) then it would
        // need to blend each new pixel with whatever it's drawn on top of.
        // 请注意，此代码仅适用于不透明的颜色。如果我们添加透明度（通过读取 opacity 属性，
        // 或在 CSS 解析器中添加对 rgba() 值的支持），那么它需要将每个新像素与它上面绘制的任何内容混合
    }
}

// Now we can put everything together in the paint function, which builds a display list and then rasterizes it to a canvas:
// 现在我们可以将所有内容放在paint函数中，它会构建一个显示列表，然后将其光栅化到画布上

/// Paint a tree of LayoutBoxes to an array of pixels.
/// 将布局框树绘制到像素数组
pub fn paint(layout_root: &LayoutBox, bounds: Rect) -> Canvas {
    let display_list = build_display_list(layout_root);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    for item in display_list {
        canvas.paint_item(&item);
    }

    canvas
}