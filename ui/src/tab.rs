use transport::vo::Screen;

pub struct TabInfo {
    id: String,
    title: String,
    zoom: f64,
    mode: Mode,
    screens: Vec<Screen>,
    saved: bool
}

impl TabInfo {
    pub fn new(id: String, title: String, zoom: f64, screens: Vec<Screen>) -> TabInfo {
        TabInfo { id, title, zoom, mode: Mode::Design, screens, saved: false}
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
}

pub enum Mode {
    Design,
    Prototype,
    VersionControl
}
