use super::{
    close_project_dialog, delete_project_dialog, page::Page, unicode_symbols, ProjectHandle,
};
use crate::{
    hardware_detect::GpuAccelerationMode,
    model::project::{ClosedHandle, Manager, OpenHandle, UntitledHandle, PROJECT_FILE_EXT},
    ui::error_dialog,
};
use eframe::{CreationContext, Frame};
use egui::{
    menu, Button, CentralPanel, Context, Image, Key, KeyboardShortcut, Modifiers, SidePanel,
    TopBottomPanel, Ui, ViewportCommand, Visuals, WidgetText,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{mem, ops::DerefMut};
use strum::AsRefStr;

macro_rules! shortcut {
    ( $name:ident, $modifiers:expr, $key:expr) => {
        paste::paste! {
            const [<$name _SHORTCUT>]: egui::KeyboardShortcut = egui::KeyboardShortcut::new($modifiers, $key);
        }
    };
}

shortcut!(NEW_PROJECT, Modifiers::COMMAND, Key::N);
shortcut!(OPEN_PROJECT, Modifiers::COMMAND, Key::O);
shortcut!(OPEN_WORKSPACE, Modifiers::COMMAND, Key::W);
shortcut!(SAVE_PROJECT, Modifiers::COMMAND, Key::S);
shortcut!(
    SAVE_AS_PROJECT,
    Modifiers::COMMAND.plus(Modifiers::SHIFT),
    Key::S
);
shortcut!(TOGGLE_PROJECT_PANEL, Modifiers::COMMAND, Key::P);

#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    page: Page,
    show_disjoint_dialog: bool,
}

impl Default for PageData {
    fn default() -> Self {
        Self {
            page: Page::drawing(),
            show_disjoint_dialog: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Theme {
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    project_manager: Manager<PageData>,
    project_handle: ProjectHandle,
    close_project_dialog: Option<close_project_dialog::State>,
    delete_project_dialog: Option<delete_project_dialog::State>,
    error: Option<String>,
    theme: Theme,
    show_project_panel: bool,
    #[serde(skip)]
    gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>,
    #[serde(skip)]
    gpu_mode: GpuAccelerationMode,
}

impl Default for App {
    fn default() -> Self {
        let manager = Manager::default().expect("Unable to create project manager");
        let project_handle = Self::next_untitled_or_open_project(&manager);
        Self {
            project_handle,
            project_manager: manager,
            close_project_dialog: None,
            delete_project_dialog: None,
            error: None,
            theme: Theme::default(),
            show_project_panel: true,
            gpu_pipeline: None,
            gpu_mode: GpuAccelerationMode::CpuOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
enum LabelButton {
    Save,
    Close,
    Delete,
}

impl LabelButton {
    const fn symbol(&self) -> &'static str {
        match self {
            LabelButton::Save => unicode_symbols::FILE,
            LabelButton::Close => unicode_symbols::CROSS,
            LabelButton::Delete => unicode_symbols::TRASH_CAN,
        }
    }
}

struct LabelResponse<H, const N: usize> {
    project_handle: H,
    selected: bool,
    buttons: [(LabelButton, bool); N],
}

impl<H, const N: usize> LabelResponse<H, N> {
    fn has_some_value(&self) -> bool {
        self.selected || self.buttons.iter().any(|(_, clicked)| *clicked)
    }

    fn is_button_clicked(&self, button: LabelButton) -> bool {
        self.buttons
            .iter()
            .find_map(|(b, c)| (button == *b).then_some(*c))
            .unwrap_or_default()
    }
}

fn project_label<H, const N: usize>(
    ui: &mut Ui,
    project_handle: H,
    is_selected: bool,
    text: impl Into<WidgetText>,
    buttons: [LabelButton; N],
) -> LabelResponse<H, N> {
    let text: WidgetText = text.into();
    let mut selected = false;
    let mut button_responses = Vec::new();

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let icon = if is_selected { "🔹" } else { "📁" };
        ui.label(icon);
        
        let response = ui.selectable_label(is_selected, text);
        selected = response.clicked();
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for button in buttons {
                let btn_resp = ui.small_button(button.symbol())
                    .on_hover_text(button.as_ref());
                button_responses.push((button, btn_resp.clicked()));
            }
        });
    });

    let mut buttons_final = [(LabelButton::Save, false); N];
    for (i, (b, c)) in button_responses.into_iter().rev().enumerate() {
        buttons_final[i] = (b, c);
    }

    LabelResponse {
        project_handle,
        selected,
        buttons: buttons_final,
    }
}

macro_rules! delete_project {
    ($($type:ident),*) => {
        paste::paste! {
            $(
                fn [<delete_ $type:lower _project>](&mut self, handle: &[<$type Handle>]) {
                    let needs_handle_update = self.project_handle == ProjectHandle::$type(*handle);
                    let result = self.project_manager.[<delete_ $type:lower _project>](handle);
                    match result {
                        Ok(()) => {
                            if needs_handle_update {
                                self.project_handle =
                                    Self::next_untitled_or_open_project(&self.project_manager);
                            }
                        }
                        Err(err) => self.error = Some(err.to_string()),
                    }
                }
            )*
        }
    };
}

impl App {
    pub fn new(cc: &CreationContext<'_>, gpu_mode: GpuAccelerationMode) -> Self {
        puffin::profile_function!();
        let mut app: Self = cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();
        app.gpu_mode = gpu_mode;
        app.update_theme(&cc.egui_ctx);
        
        // Initialize GPU pipeline if not already present
        if app.gpu_pipeline.is_none() {
            app.gpu_pipeline = pollster::block_on(cpd_wgpu::ComputePipeline::new())
                .map(std::sync::Arc::new);
            if app.gpu_pipeline.is_some() {
                log::info!("GPU acceleration initialized successfully.");
            } else {
                log::warn!("GPU acceleration not available on this system.");
            }
        }
        
        app
    }

    fn add_menu(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| self.add_file_menu_items(ui));
            ui.menu_button("View", |ui| self.add_view_menu_items(ui));
            if let Some(page_data) = self.page_data() {
                page_data.page = mem::take(&mut page_data.page).add_menu_items(ui);
            }
            ui.centered_and_justified(|ui| {
                ui.label(format!("Workspace - {}", self.project_manager.workspace()));
            });
        });
    }

    fn update_theme(&mut self, ctx: &Context) {
        let mut visuals = match self.theme {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        };

        // --- Custom Color Palette ---
        let accent_color = egui::Color32::from_rgb(79, 70, 229); // Indigo 600
        let _bg_dark = egui::Color32::from_rgb(12, 12, 14);
        let _panel_dark = egui::Color32::from_rgb(18, 18, 22);
        let _surface_dark = egui::Color32::from_rgb(24, 24, 28);
        
        if self.theme == Theme::Dark {
            // Deep cosmic black with a very subtle indigo undertone
            visuals.window_fill        = egui::Color32::from_rgb(10, 10, 16);
            visuals.panel_fill         = egui::Color32::from_rgb(8, 8, 14);
            visuals.faint_bg_color     = egui::Color32::from_rgb(18, 18, 28);
            visuals.extreme_bg_color   = egui::Color32::from_rgb(4, 4, 8);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(16, 16, 24);
            visuals.widgets.inactive.bg_fill       = egui::Color32::from_rgb(22, 22, 32);
            visuals.widgets.hovered.bg_fill        = egui::Color32::from_rgb(32, 30, 48);
            visuals.widgets.active.bg_fill         = egui::Color32::from_rgb(44, 40, 64);
        } else {
            visuals.window_fill = egui::Color32::WHITE;
            visuals.panel_fill  = egui::Color32::from_rgb(246, 248, 252); // Slate 50 with blue tint
            visuals.faint_bg_color = egui::Color32::from_rgb(238, 242, 250); // Slate 100 blue
        }

        visuals.selection.bg_fill = accent_color.linear_multiply(0.8);

        // --- Rounding & Spacing ---
        let rounding_val = 10.0;
        visuals.window_rounding = egui::Rounding::same(rounding_val);
        visuals.menu_rounding   = egui::Rounding::same(rounding_val - 2.0);

        let widget_rounding = egui::Rounding::same(8.0);
        visuals.widgets.noninteractive.rounding = widget_rounding;
        visuals.widgets.inactive.rounding       = widget_rounding;
        visuals.widgets.hovered.rounding        = widget_rounding;
        visuals.widgets.active.rounding         = widget_rounding;
        visuals.widgets.open.rounding           = widget_rounding;

        // --- Strokes & Shadows ---
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 40));
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 20));
        visuals.widgets.hovered.bg_stroke  = egui::Stroke::new(1.5, accent_color);

        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 20.0),
            blur: 48.0,
            spread: -8.0,
            color: egui::Color32::from_rgba_premultiplied(
                0, 0, 0, if self.theme == Theme::Dark { 200 } else { 30 }
            ),
        };

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(20.0);
        style.spacing.interact_size.y = 24.0;
        
        // --- Typography ---
        // Increase default font size slightly for readability
        for font_id in style.text_styles.values_mut() {
            font_id.size += 1.0;
        }
        
        ctx.set_style(style);
    }

    fn add_file_menu_items(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        self.add_new_project_menu_button(ui);
        self.add_open_project_menu_button(ui);
        self.add_open_workspace_menu_button(ui);
        self.add_open_recents_menu_button(ui);
        self.add_save_menu_buttons(ui);
        self.add_clear_data_menu_button(ui);
        if ui.button("Quit").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn button_with_shortcut_clicked(
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        shortcut: &KeyboardShortcut,
    ) -> bool {
        ui.add(Button::new(text).shortcut_text(ui.ctx().format_shortcut(shortcut)))
            .clicked()
    }

    fn add_new_project_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !Self::button_with_shortcut_clicked(ui, "New Project", &NEW_PROJECT_SHORTCUT) {
            return;
        }
        ui.close_menu();
        self.create_untitled_project();
    }

    fn create_untitled_project(&mut self) {
        puffin::profile_function!();
        self.project_manager.create_untitled_project();
        self.project_handle = ProjectHandle::Untitled(
            self.project_manager
                .untitled_project_handles()
                .last()
                .expect("At least one untitled project handle should exist"),
        );
    }

    fn add_open_project_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !Self::button_with_shortcut_clicked(ui, "Open Project", &OPEN_PROJECT_SHORTCUT) {
            return;
        }
        ui.close_menu();
        self.open_project();
    }

    fn open_project(&mut self) {
        puffin::profile_function!();
        let opt = FileDialog::new()
            .add_filter("Project", &[PROJECT_FILE_EXT.to_string_lossy()])
            .set_directory(self.project_manager.workspace().path())
            .pick_file();
        let Some(path) = opt else {
            return;
        };
        match self.project_manager.open_project(path) {
            Ok(handle) => self.project_handle = ProjectHandle::Open(handle),
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn add_open_workspace_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !Self::button_with_shortcut_clicked(ui, "Open Workspace", &OPEN_WORKSPACE_SHORTCUT) {
            return;
        }
        ui.close_menu();
        self.open_workspace();
    }

    fn open_workspace(&mut self) {
        puffin::profile_function!();
        let opt = FileDialog::new()
            .set_directory(self.project_manager.workspace().path())
            .pick_folder();
        let Some(path) = opt else {
            return;
        };
        if let Err(err) = self.project_manager.set_workspace(path) {
            self.error = Some(err.to_string());
        }
    }

    fn add_open_recents_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !self.project_manager.has_recent_projects()
            && !self.project_manager.has_recent_workspaces()
        {
            return;
        }
        ui.menu_button("Open Recent", |ui| {
            self.add_recent_projects_in_menu(ui);
            self.add_recent_workspaces_in_menu(ui);
            if !ui.button("Clear Recents").clicked() {
                return;
            }
            ui.close_menu();
            self.project_manager.clear_recents();
        });
    }

    fn add_recent_projects_in_menu(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !self.project_manager.has_recent_projects() {
            return;
        }
        let opt = self
            .project_manager
            .recent_project_handles()
            .filter_map(|handle| {
                ui.button(self.project_manager[handle].name().to_string_lossy())
                    .clicked()
                    .then_some(*handle)
            })
            .next();
        if let Some(handle) = opt {
            ui.close_menu();
            match self.project_manager.open_recent_project(&handle) {
                Ok(handle) => self.project_handle = ProjectHandle::Open(handle),
                Err(err) => self.error = Some(err.to_string()),
            }
            return;
        };
        ui.separator();
    }

    fn add_recent_workspaces_in_menu(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !self.project_manager.has_recent_workspaces() {
            return;
        }
        let opt = self
            .project_manager
            .recent_workspaces()
            .filter_map(|handle| {
                ui.button(self.project_manager[handle].to_string())
                    .clicked()
                    .then_some(*handle)
            })
            .next();
        if let Some(handle) = opt {
            ui.close_menu();
            match self.project_manager.open_recent_workspace(&handle) {
                Ok(()) => {}
                Err(err) => self.error = Some(err.to_string()),
            }
            return;
        };
        ui.separator();
    }

    fn add_save_menu_buttons(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        match self.project_handle {
            ProjectHandle::Open(handle) => {
                if Self::button_with_shortcut_clicked(ui, "Save", &SAVE_PROJECT_SHORTCUT) {
                    ui.close_menu();
                    self.save_open_project(&handle)
                }

                if Self::button_with_shortcut_clicked(
                    ui,
                    const_format::formatcp!("Save As{}", unicode_symbols::ELLIPSIS),
                    &SAVE_AS_PROJECT_SHORTCUT,
                ) {
                    ui.close_menu();
                    self.save_open_project_at_path(&handle);
                }
            }
            ProjectHandle::Untitled(handle) => {
                if Self::button_with_shortcut_clicked(ui, "Save", &SAVE_PROJECT_SHORTCUT) {
                    ui.close_menu();
                    self.save_untitled_project(&handle);
                }
            }
            _ => {}
        }
    }

    fn add_view_menu_items(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        self.add_theme_menu_button(ui);
        self.add_project_panel_toggle(ui);
    }

    fn add_theme_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        ui.menu_button("Theme", |ui| {
            let theme = self.theme;
            ui.selectable_value(
                &mut self.theme,
                Theme::Light,
                const_format::formatcp!("{} Light mode", unicode_symbols::SUN),
            );
            ui.selectable_value(
                &mut self.theme,
                Theme::Dark,
                const_format::formatcp!("{} Dark mode", unicode_symbols::MOON),
            );
            if self.theme != theme {
                self.update_theme(ui.ctx());
            }
        });
    }

    fn add_project_panel_toggle(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !self.has_projects_to_show() {
            return;
        }
        let response = ui.add(
            Button::new(if self.show_project_panel {
                "Hide project panel"
            } else {
                "Show project pane"
            })
            .shortcut_text(ui.ctx().format_shortcut(&TOGGLE_PROJECT_PANEL_SHORTCUT)),
        );
        if response.clicked() {
            self.toggle_project_panel();
        }
    }

    fn toggle_project_panel(&mut self) {
        self.show_project_panel = !self.show_project_panel;
    }

    fn save_open_project(&mut self, handle: &OpenHandle) {
        puffin::profile_function!();
        match self
            .project_manager
            .get_open_project_mut(handle)
            .expect("Handle is valid")
            .save()
        {
            Ok(()) => {}
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
    }

    fn save_open_project_at_path(&mut self, handle: &OpenHandle) {
        puffin::profile_function!();
        let opt = FileDialog::new()
            .add_filter("Project", &[PROJECT_FILE_EXT.to_string_lossy()])
            .set_directory(self.project_manager.workspace().path())
            .set_file_name(self.project_manager[handle].name().to_string_lossy())
            .save_file();
        let Some(path) = opt else {
            return;
        };
        let needs_handle_update = self.project_handle == ProjectHandle::Open(*handle);
        match self.project_manager.save_open_project_in_path(handle, path) {
            Ok(handle) => {
                if needs_handle_update {
                    self.project_handle = ProjectHandle::Open(handle);
                }
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
    }

    fn save_untitled_project(&mut self, handle: &UntitledHandle) -> Option<OpenHandle> {
        puffin::profile_function!();
        let opt = FileDialog::new()
            .add_filter("Project", &[PROJECT_FILE_EXT.to_string_lossy()])
            .set_directory(self.project_manager.workspace().path())
            .add_filter("Project", &[PROJECT_FILE_EXT.to_string_lossy()])
            .save_file();
        let path = opt?;
        let needs_handle_update = self.project_handle == ProjectHandle::Untitled(*handle);
        match self.project_manager.save_untitled_project(handle, path) {
            Ok(handle) => {
                if needs_handle_update {
                    self.project_handle = ProjectHandle::Open(handle);
                }
                Some(handle)
            }
            Err(err) => {
                self.error = Some(err.to_string());
                None
            }
        }
    }

    fn add_clear_data_menu_button(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        if !ui.button("Clear Data").clicked() {
            return;
        }
        ui.close_menu();
        *self = App::default();
    }

    fn open_closed_project(&mut self, handle: &ClosedHandle) {
        puffin::profile_function!();
        match self.project_manager.open_closed_project(handle) {
            Ok(handle) => self.project_handle = ProjectHandle::Open(handle),
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn refresh_workspace(&mut self) {
        puffin::profile_function!();
        if let Err(err) = self.project_manager.refresh_workspace() {
            self.error = Some(err.to_string());
        }
    }

    fn add_projects(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("WORKSPACE").small().strong().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(unicode_symbols::REFRESH).clicked() {
                    self.refresh_workspace();
                }
            });
        });
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            
            if self.project_manager.has_untitled_projects() {
                egui::CollapsingHeader::new("Untitled")
                    .default_open(true)
                    .show(ui, |ui| self.add_untitled_projects(ui));
            }

            if self.project_manager.has_open_projects() {
                egui::CollapsingHeader::new("Open Projects")
                    .default_open(true)
                    .show(ui, |ui| self.add_open_projects(ui));
            }

            if self.project_manager.has_closed_projects() {
                egui::CollapsingHeader::new("Recent")
                    .default_open(false)
                    .show(ui, |ui| self.add_closed_projects(ui));
            }
        });
    }

    fn add_untitled_projects(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let opt = self
            .project_manager
            .untitled_project_handles()
            .map(|handle| {
                project_label(
                    ui,
                    handle,
                    self.project_handle == ProjectHandle::Untitled(handle),
                    self.project_manager[handle].name().to_string_lossy(),
                    [LabelButton::Save, LabelButton::Close],
                )
            })
            .find(LabelResponse::has_some_value);
        let Some(response) = opt else {
            return;
        };
        if response.selected {
            self.project_handle = ProjectHandle::Untitled(response.project_handle);
        }
        if response.is_button_clicked(LabelButton::Close) {
            self.close_project_dialog = Some(close_project_dialog::State::new(
                self.project_manager[response.project_handle].name().clone(),
                ProjectHandle::Untitled(response.project_handle),
            ));
        } else if response.is_button_clicked(LabelButton::Save) {
            self.save_untitled_project(&response.project_handle);
        }
    }

    fn add_open_projects(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let opt = self
            .project_manager
            .open_project_handles()
            .map(|handle| {
                project_label(
                    ui,
                    *handle,
                    self.project_handle == ProjectHandle::Open(*handle),
                    self.project_manager[handle].name().to_string_lossy(),
                    [LabelButton::Save, LabelButton::Close, LabelButton::Delete],
                )
            })
            .find(LabelResponse::has_some_value);
        let Some(response) = opt else {
            return;
        };
        if response.selected {
            self.project_handle = ProjectHandle::Open(response.project_handle);
        }
        if response.is_button_clicked(LabelButton::Delete) {
            self.delete_project_dialog = Some(delete_project_dialog::State::new(
                self.project_manager[&response.project_handle]
                    .name()
                    .clone(),
                ProjectHandle::Open(response.project_handle),
            ));
        } else if response.is_button_clicked(LabelButton::Close) {
            self.close_project_dialog = Some(close_project_dialog::State::new(
                self.project_manager[&response.project_handle]
                    .name()
                    .clone(),
                ProjectHandle::Open(response.project_handle),
            ));
        } else if response.is_button_clicked(LabelButton::Save) {
            self.save_open_project(&response.project_handle);
        }
    }

    fn add_closed_projects(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let opt = self
            .project_manager
            .closed_project_handles()
            .map(|handle| {
                project_label(
                    ui,
                    *handle,
                    self.project_handle == ProjectHandle::Closed(*handle),
                    self.project_manager[handle].name().to_string_lossy(),
                    [LabelButton::Delete],
                )
            })
            .find(LabelResponse::has_some_value);
        let Some(response) = opt else {
            return;
        };
        if response.is_button_clicked(LabelButton::Delete) {
            self.delete_project_dialog = Some(delete_project_dialog::State::new(
                self.project_manager[&response.project_handle]
                    .name()
                    .clone(),
                ProjectHandle::Closed(response.project_handle),
            ));
        } else if response.selected {
            self.open_closed_project(&response.project_handle);
        }
    }

    fn next_untitled_or_open_project<D>(project_manager: &Manager<D>) -> ProjectHandle
    where
        D: Default + Serialize + for<'de> Deserialize<'de>,
    {
        project_manager
            .untitled_project_handles()
            .map(ProjectHandle::Untitled)
            .chain(
                project_manager
                    .open_project_handles()
                    .copied()
                    .map(ProjectHandle::Open),
            )
            .next()
            .unwrap_or_default()
    }

    fn add_welcome(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let mut action = None;

        ui.vertical_centered(|ui| {
            ui.add_space(ui.max_rect().height() / 6.0);
            
            // --- Hero Section ---
            ui.add(
                Image::new(egui::include_image!("../assets/simulator-icon.svg"))
                    .tint(egui::Color32::from_rgb(99, 102, 241)) // Indigo 500
                    .maintain_aspect_ratio(true)
                    .max_width(120.0),
            );
            
            ui.add_space(24.0);
            ui.label(egui::RichText::new("3D DPM Simulator").strong().size(32.0));
            ui.label(egui::RichText::new("Discrete Particle Method & Continuum Interaction Engine").weak().size(16.0));
            
            ui.add_space(48.0);
            
            // --- Quick Actions ---
            ui.columns(3, |columns| {
                if self.welcome_card(&mut columns[0], "➕", "New Project", "Start a fresh simulation").clicked() {
                    action = Some("new");
                }
                if self.welcome_card(&mut columns[1], "📂", "Open Project", "Load existing .dpm file").clicked() {
                    action = Some("open");
                }
                if self.welcome_card(&mut columns[2], "🏠", "Workspace", "Set your working directory").clicked() {
                    action = Some("workspace");
                }
            });

            ui.add_space(60.0);
            ui.label(egui::RichText::new("Press Ctrl+N to get started immediately").small().weak());
        });

        match action {
            Some("new") => self.create_untitled_project(),
            Some("open") => self.open_project(),
            Some("workspace") => self.open_workspace(),
            _ => {}
        }
    }

    fn welcome_card(&self, ui: &mut Ui, icon: &str, title: &str, subtitle: &str) -> egui::Response {
        let card_width = 180.0;
        let card_height = 140.0;
        
        let (rect, response) = ui.allocate_at_least(egui::vec2(card_width, card_height), egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let fill = if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().widgets.noninteractive.bg_fill
            };
            
            ui.painter().rect(
                rect,
                12.0,
                fill,
                egui::Stroke::new(1.0, visuals.bg_stroke.color)
            );
            
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(16.0)).layout(egui::Layout::top_down(egui::Align::Center)));
            child_ui.label(egui::RichText::new(icon).size(32.0));
            child_ui.add_space(8.0);
            child_ui.label(egui::RichText::new(title).strong().size(16.0));
            child_ui.add_space(4.0);
            child_ui.label(egui::RichText::new(subtitle).small().weak());
        }
        
        response
    }

    fn close_open_project(&mut self, handle: &OpenHandle) {
        puffin::profile_function!();
        self.project_manager.close_open_project(handle);
        self.project_handle = Self::next_untitled_or_open_project(&self.project_manager);
    }

    delete_project!(Open, Closed);

    fn show_close_project_dialog(&mut self, ctx: &Context) {
        puffin::profile_function!();
        let Some(state) = &mut self.close_project_dialog else {
            return;
        };
        use close_project_dialog::Response;
        match close_project_dialog::show(state, ctx) {
            Response::Noop => {}
            Response::Save(handle) => match handle {
                ProjectHandle::Invalid | ProjectHandle::Recent(_) | ProjectHandle::Closed(_) => {
                    unreachable!()
                }
                ProjectHandle::Open(handle) => {
                    self.close_project_dialog = None;
                    self.close_open_project(&handle);
                }
                ProjectHandle::Untitled(handle) => {
                    self.close_project_dialog = None;
                    if let Some(handle) = self.save_untitled_project(&handle) {
                        self.close_open_project(&handle);
                    }
                }
            },
            Response::Discard(handle) => match handle {
                ProjectHandle::Invalid | ProjectHandle::Recent(_) | ProjectHandle::Closed(_) => {
                    unreachable!()
                }
                ProjectHandle::Open(handle) => {
                    self.close_project_dialog = None;
                    self.close_open_project(&handle);
                }
                ProjectHandle::Untitled(handle) => {
                    self.close_project_dialog = None;
                    self.project_manager.discard_untitled_project(&handle);
                    self.project_handle =
                        Self::next_untitled_or_open_project(&self.project_manager);
                }
            },
            Response::Cancel => {
                self.close_project_dialog = None;
            }
        }
    }

    fn show_delete_project_dialog(&mut self, ctx: &Context) {
        puffin::profile_function!();
        let Some(state) = &mut self.delete_project_dialog else {
            return;
        };
        use delete_project_dialog::Response;
        match delete_project_dialog::show(state, ctx) {
            Response::Noop => {}
            Response::Delete(handle) => match handle {
                ProjectHandle::Invalid | ProjectHandle::Recent(_) | ProjectHandle::Untitled(_) => {
                    unreachable!()
                }
                ProjectHandle::Open(handle) => {
                    self.delete_project_dialog = None;
                    self.delete_open_project(&handle);
                }
                ProjectHandle::Closed(handle) => {
                    self.delete_project_dialog = None;
                    self.delete_closed_project(&handle);
                }
            },
            Response::Cancel => {
                self.delete_project_dialog = None;
            }
        }
    }

    fn add_shortcuts(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        macro_rules! shortcut {
            ( $shortcut:expr, $command:expr ) => {
                if super::consume_shortcut(ui, &$shortcut) {
                    $command;
                }
            };
        }
        shortcut!(NEW_PROJECT_SHORTCUT, self.create_untitled_project());
        shortcut!(OPEN_PROJECT_SHORTCUT, self.open_project());
        shortcut!(OPEN_WORKSPACE_SHORTCUT, self.open_workspace());

        match self.project_handle {
            ProjectHandle::Open(handle) => {
                shortcut!(SAVE_PROJECT_SHORTCUT, self.save_open_project(&handle));
                shortcut!(
                    SAVE_AS_PROJECT_SHORTCUT,
                    self.save_open_project_at_path(&handle)
                );
            }
            ProjectHandle::Untitled(handle) => {
                shortcut!(SAVE_PROJECT_SHORTCUT, {
                    self.save_untitled_project(&handle);
                });
            }
            _ => {}
        }

        shortcut!(TOGGLE_PROJECT_PANEL_SHORTCUT, self.toggle_project_panel());
    }

    fn page_data(&mut self) -> Option<&mut PageData> {
        match self.project_handle {
            ProjectHandle::Open(handle) => Some(
                self.project_manager
                    .get_open_project_mut(&handle)
                    .expect("Handle is valid")
                    .state_mut()
                    .deref_mut(),
            ),
            ProjectHandle::Untitled(handle) => {
                Some(self.project_manager[handle].state_mut().deref_mut())
            }
            _ => None,
        }
    }

    fn add_contents(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let gpu_pipeline = self.gpu_pipeline.clone();
        let Some(page_data) = self.page_data() else {
            ui.centered_and_justified(|ui| {
                ui.heading("Select a project from the side panel!");
            });
            return;
        };

        if page_data.show_disjoint_dialog
            && error_dialog::show(
                "There are disjoint shapes, please merge them or remove",
                ui.ctx(),
            )
            .closed()
        {
            page_data.show_disjoint_dialog = false;
        }

        ui.vertical_centered_justified(|ui| {
            page_data.page = mem::take(&mut page_data.page).add_contents(ui, gpu_pipeline);
        });
    }

    fn has_projects_to_show(&self) -> bool {
        self.project_manager.has_untitled_projects()
            || self.project_manager.has_open_projects()
            || self.project_manager.has_closed_projects()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        puffin::profile_function!();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        puffin::profile_function!();
        egui_extras::install_image_loaders(ctx);
        let change_theme = ctx.style().visuals.dark_mode && self.theme != Theme::Dark;
        if change_theme {
            self.update_theme(ctx);
        }
        self.show_close_project_dialog(ctx);
        self.show_delete_project_dialog(ctx);
        TopBottomPanel::top("top_panel").show(ctx, |ui| self.add_menu(ui));
        if self.has_projects_to_show() && self.show_project_panel {
            SidePanel::left("projects_panel").show(ctx, |ui| self.add_projects(ui));
        }
        CentralPanel::default().show(ctx, |ui| {
            self.add_shortcuts(ui);

            if self.has_projects_to_show() {
                self.add_contents(ui);
            } else {
                self.add_welcome(ui);
            }

            self.error = self
                .error
                .take()
                .filter(|err| !error_dialog::show(err, ui.ctx()).closed());
        });
    }
}
