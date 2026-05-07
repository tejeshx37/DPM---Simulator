use crate::model::RefreshToken;
use egui::Context;

#[derive(Debug, Default)]
pub enum ContextWrapper {
    #[default]
    None,
    Context(Context),
}

impl From<&Context> for ContextWrapper {
    fn from(value: &Context) -> Self {
        Self::Context(value.clone())
    }
}

impl RefreshToken for ContextWrapper {
    fn refresh(&self) {
        match self {
            ContextWrapper::None => {}
            ContextWrapper::Context(ctx) => ctx.request_repaint(),
        }
    }
}
