use egui::{Frame, Id, InnerResponse, Margin, TopBottomPanel, Ui};

pub fn show<F, R>(id: impl Into<Id>, ui: &mut Ui, add_contents: F) -> InnerResponse<R>
where
    F: FnOnce(&mut Ui) -> R,
{
    TopBottomPanel::bottom(id)
        .frame(Frame::default().inner_margin(Margin {
            left: 0.0,
            right: 0.0,
            top: 12.0,
            bottom: 0.0,
        }))
        .show_inside(ui, add_contents)
}
