use euclid::Transform2D;

use super::{
    coordinates::{canvas_to_screen, screen_to_canvas, CanvasPoint, CanvasScope, ScreenScope},
    IndexedItem, Item,
};

pub enum ScreenItems {
    Items(Vec<Item>),
    Proxy,
}

pub enum IndexedScreenItems {
    Items(Vec<IndexedItem>),
    Proxy,
}

pub struct IndexedScreenWithChild {
    pub meta: Screen,
    pub items: IndexedScreenItems,
}

impl IndexedScreenWithChild {
    pub fn remove_indexes(&self) -> ScreenWithChild {
        let non_indexed_items = match &self.items {
            IndexedScreenItems::Items(items) => ScreenItems::Items(
                items
                    .into_iter()
                    .map(|i| match i {
                        IndexedItem::Line { line, .. } => Item::Line(line.clone()),
                        IndexedItem::Rectangle { rectangle, ..} => Item::Rectangle(rectangle.clone())
                    })
                    .collect::<Vec<_>>(),
            ),
            IndexedScreenItems::Proxy => ScreenItems::Proxy,
        };
        ScreenWithChild {
            meta: self.meta.clone(),
            items: non_indexed_items,
        }
    }
}

pub struct ScreenWithChild {
    pub meta: Screen,
    pub items: ScreenItems,
}

#[derive(Clone)]
pub struct Screen {
    min: CanvasPoint,
    /// Width in nano meters
    width: f64,
    /// Height in nano meters
    height: f64,
    /// Resolution in pixels
    resolution: (u32, u32),
    /// Title of the screen
    title: String,
}

impl Screen {
    pub fn new(
        min: CanvasPoint,
        width: f64,
        height: f64,
        resolution: (u32, u32),
        title: String,
    ) -> Screen {
        Screen {
            min,
            width,
            height,
            resolution,
            title,
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    /// Returning the pixel count per one centimeter
    pub fn get_ppcm(&self) -> f32 {
        ((((self.resolution.0 as f64) * 10000.0) / (self.width * 2.0)) as f32)
            + ((((self.resolution.1 as f64) * 10000.0) / (self.height * 2.0)) as f32)
    }

    /// Returning the transformation for inner coordinate system
    pub fn get_inner_transformation(&self) -> Transform2D<f64, ScreenScope, CanvasScope> {
        screen_to_canvas(self.get_ppcm(), self.min.x, self.min.y)
    }

    /// Returning the transformation for outer coordinate system
    pub fn get_outer_transformation(&self) -> Transform2D<f64, CanvasScope, ScreenScope> {
        canvas_to_screen(self.get_ppcm(), self.min.x, self.min.y)
    }

    /// Returning the point of top left corner
    pub fn tl(&self) -> CanvasPoint {
        self.min
    }

    /// Returning the point of top right corner
    pub fn tr(&self) -> CanvasPoint {
        CanvasPoint::new(self.min.x + self.width, self.min.y)
    }

    pub fn bl(&self) -> CanvasPoint {
        CanvasPoint::new(self.min.x, self.min.y + self.height)
    }

    pub fn br(&self) -> CanvasPoint {
        CanvasPoint::new(self.min.x + self.width, self.min.y + self.height)
    }
}
