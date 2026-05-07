use egui::{TextEdit, Ui};

pub struct Field<'value> {
    pub name: &'static str,
    pub value: &'value mut String,
}

pub fn single_line_double_input_field<'field>(
    ui: &mut Ui,
    first: Field<'field>,
    second: Field<'field>,
) {
    let item_spacing = ui.spacing().item_spacing.x;
    ui.horizontal(|ui| {
        let width = ui.label(first.name).rect.width();
        TextEdit::singleline(first.value)
            .desired_width((ui.available_width() / 2.0) - (2.0 * item_spacing) - width)
            .show(ui);
    });
    ui.horizontal(|ui| {
        let width = ui.label(second.name).rect.width();
        TextEdit::singleline(second.value)
            .desired_width(ui.available_width() + item_spacing - width)
            .show(ui);
    });
}

pub mod ok_cancel {
    use egui::{Key, Ui};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Response {
        Ok,
        Cancel,
        #[default]
        Noop,
    }

    pub fn buttons(ui: &mut Ui) -> Response {
        if ui.input(|state| state.key_pressed(Key::Enter)) || ui.button("Ok").clicked() {
            Response::Ok
        } else if ui.button("Cancel").clicked() {
            Response::Cancel
        } else {
            Response::Noop
        }
    }
}
