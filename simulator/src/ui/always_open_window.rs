use egui::{Context, Ui, WidgetText, Window};

pub struct AlwaysOpenWindow<'a>(Window<'a>);

impl<'a> AlwaysOpenWindow<'a> {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self(Window::new(title).collapsible(false))
    }

    pub fn resizable(self, resizable: bool) -> Self {
        Self(self.0.resizable(resizable))
    }

    pub fn default_width(self, default_width: f32) -> Self {
        Self(self.0.default_width(default_width))
    }

    pub fn default_height(self, default_height: f32) -> Self {
        Self(self.0.default_height(default_height))
    }

    #[allow(dead_code)]
    pub fn max_height(self, max_height: f32) -> Self {
        Self(self.0.max_height(max_height))
    }

    pub fn show<R>(self, ctx: &Context, add_contents: impl FnMut(&mut Ui) -> R) -> R {
        self.0
            .show(ctx, add_contents)
            .expect("Window is always open, so this should not be None")
            .inner
            .expect("Window is not collapsible, so this should not be None.")
    }
}
