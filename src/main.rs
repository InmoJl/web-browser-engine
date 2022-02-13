extern crate image;

use std::default::Default;
use std::fs;

mod html;
mod dom;
mod css;
mod style;
mod layout;
mod painting;

fn main() {

    // 取得当前项目路径
    let mut path = std::env::current_dir().unwrap();
    path.push("examples/index");

    // 解析 html
    path.set_extension("html");
    let html_source = read_source(path.to_str().unwrap());
    let html_tree = html::parse(html_source);
    // println!("{:#?}", html_tree);

    // 解析 css
    path.set_extension("css");
    let css_source = read_source(path.to_str().unwrap());
    let stylesheet_tree  = css::parse(css_source);
    // println!("{:#?}", stylesheet_tree );

    // 将 html-tree 与 css-tree 合并为 style-tree
    let style_tree = style::style_tree(&html_tree, &stylesheet_tree);
    // println!("{:#?}", style_root);

    // 预设样式，初始配置
    let initial_containing_block = layout::Dimensions {
        content: layout::Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        padding: Default::default(),
        border: Default::default(),
        margin: Default::default()
    };
    // 构建 layout-tree
    let layout_tree = layout::layout_tree(&style_tree, initial_containing_block);
    // println!("{:#?}", layout_tree);

    // 创建绘制画布 栅格化
    let canvas = painting::paint(&layout_tree, initial_containing_block.content);
    // println!("{:#?}", canvas);

    // 将画布保存为图片
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<image::Rgba<u8>> = unsafe {
        std::mem::transmute(canvas.pixels)
    };
    let img = image::ImageBuffer::from_fn(
        w,
        h,
        Box::new(|x: u32, y: u32| buffer[(y * w + x) as usize])
    );
    let result = img.save("test.png");

    match result {
        Ok(_) => println!("成功"),
        Err(_) => println!("失败")
    }
}

fn read_source(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(source) => source,
        Err(_) => panic!("读取失败")
    }
}
