#![allow(unstable)]

extern crate getopts;
extern crate image;

use std::default::Default;
use std::env::args;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use image::{ImageBuffer, RgbaImage};

mod html;
mod dom;
mod css;
mod style;
mod layout;
mod painting;

fn main() {

    // 解析命令行选项
    let mut opts = getopts::Options::new();
    opts.optopt("h", "html", "HTML document", "FILENAME");
    opts.optopt("c", "css", "CSS stylesheet", "FILENAME");
    opts.optopt("o", "output", "Output file", "FILENAME");
    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
    };

    // 解析 html
    let html_source = String::from(
        "<html>" +
            "<div class='a'>" +
                "<div class='b'>" +
                    "<div class='c'>" +
                        "<div class='d'>" +
                            "<div class='e'>" +
                                "<div class='f'>" +
                                    "<div class='g'></div>" +
                                "</div>" +
                            "</div>" +
                        "</div>" +
                    "</div>" +
                "</div>" +
            "</div>" +
        "</html>"
    );
    let html_tree = html::parse(html_source);
    // println!("{:#?}", html_tree);

    // 解析 css
    let css_source = String::from(
    "* { display: block; padding: 12px; }",
        ".a { background: #ff0000; }",
        ".b { background: #ffa500; }",
        ".c { background: #ffff00; }",
        ".d { background: #008000; }",
        ".e { background: #0000ff; }",
        ".f { background: #4b0082; }",
        ".g { background: #800080; }"
    );
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

    // 创建图片文件，将画布作为图片输出
    let filename = matches.opt_str("o").unwrap_or("output.png".to_string());
    let file = File::create(&Path::new(&filename)).unwrap();
    // 保存图片
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<image::Rgba<u8>> = unsafe {
        std::mem::transmute(canvas.pixels)
    };
    let img = image::ImageBuffer::from_fn(
        w,
        h,
        Box::new(|x: u32, y: u32| buffer[(y * w + x) as usize])
    );
    let result = img.save(filename);

    match result {
        Ok(_) => println!("成功"),
        Err(_) => println!("失败")
    }
}

fn read_source(arg_filename: Option<String>, default_filename: &str) -> String {

    let path = match arg_filename {
        Some(ref filename) => filename,
        None => default_filename,
    };

    let mut file = File::open(path).unwrap();
    let mut source = String::new();
    file.read_to_string(&mut source).unwrap();

    source
}
