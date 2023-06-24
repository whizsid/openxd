use transport::vo::Screen;

pub struct TabInfo {
    id: String,
    title: String,
    zoom: f64,
    mode: Mode,
    screens: Vec<Screen>,
    saved: bool,
    closing: bool,
}

impl TabInfo {
    pub fn new(id: String, title: String, zoom: f64, screens: Vec<Screen>) -> TabInfo {
        TabInfo { id, title, zoom, mode: Mode::Design, screens, saved: false, closing: false}
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn set_saved(&mut self, saved: bool)  {
        self.saved = saved;
    }

    pub fn saved(&self) -> bool {
        self.saved
    }

    pub fn set_closing(&mut self, closing: bool) {
        self.closing = closing;
    }

    pub fn closing(&self) -> bool {
        self.closing
    }
}

pub enum Mode {
    Design,
    Prototype,
    VersionControl
}
