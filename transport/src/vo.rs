//! Value objects that need to use in both UI and app

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point2D {
    pub x: u32,
    pub y: u32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rect2D {
    pub min: Point2D,
    pub max: Point2D,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Screen {
    pub kind: ScreenKind,
    pub rect: Rect2D,
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScreenKind {
    Proxy {
        proxy_image: String,
    },
    Full,
}
