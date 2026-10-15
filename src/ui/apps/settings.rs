//! Settings: a sidebar of sections and a content pane.

use crate::core::{Core, ServerStatus};
use crate::fieldguide::data::guide;
use crate::relay::PORT;
use crate::settings::{import_wallpaper, new_api_key, WALLPAPERS};
use crate::ui::icons;
use crate::ui::theme::{self, *};
use crate::ui::wallpaper;
use eframe::egui::{self, Align, Align2, CornerRadius, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2};

#[derive(Default)]
pub struct SettingsUi {
    pub current: usize,
    message: String,
    show_key: bool,
    confirming: bool,
    copied: Option<(String, f64)>,
}

// Add a settings section: write a pane function and add a line here.
const SECTIONS: [(&str, [u8; 3], &str); 5] = [
    ("Wallpaper", [0x2e, 0x7a, 0xb5], icons::WALLPAPER),
    ("Stream Deck", [0x1f, 0x8a, 0x6c], icons::STREAMDECK),
    ("Account", [0x5a, 0x6b, 0x85], icons::ACCOUNT),
    ("Software Update", [0x17, 0x4f, 0x84], icons::UPDATE),
    ("About", [0x1d, 0x0f, 0x5e], r#"<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7.5v.5"/>"#),
];

/// The Software Update section's place in the list.
pub const UPDATE_SECTION: usize = 3;

pub fn show(ui: &mut Ui, st: &mut SettingsUi, core: &mut Core, now: f64, radius: u8) {
    let rect = ui.max_rect();
    let narrow = rect.width() <= 520.0;
    let name = core.user.clone().unwrap_or_default();

    let (side, pane) = if narrow {
        let side = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 52.0));
        (side, Rect::from_min_max(Pos2::new(rect.left(), side.bottom()), rect.max))
    } else {
        let side = Rect::from_min_size(rect.min, Vec2::new(200.0, rect.height()));
        (side, Rect::from_min_max(Pos2::new(side.right(), rect.top()), rect.max))
    };

    // Sidebar
    let p = ui.painter().clone();
    p.rect_filled(side, CornerRadius { nw: 0, ne: 0, sw: if narrow { 0 } else { radius.saturating_sub(1) }, se: 0 }, rgba(0, 0, 0, 0.2));
    if narrow {
        p.hline(side.x_range(), side.bottom(), Stroke::new(1.0_f32, white(0.1)));
    } else {
        p.vline(side.right(), side.y_range(), Stroke::new(1.0_f32, white(0.1)));
    }
    let mut sb = ui.new_child(UiBuilder::new().max_rect(side.shrink2(Vec2::new(10.0, if narrow { 10.0 } else { 14.0 }))));
    if !narrow {
        sb.horizontal(|ui| {
            ui.add_space(8.0);
            let (r, _) = ui.allocate_exact_size(Vec2::splat(38.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 19.0, WHITE);
            let initial = name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
            ui.painter().text(r.center(), Align2::CENTER_CENTER, initial, display(16.0, 800), ACCENT);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new(theme::elide(ui, &name, &font(14.0, 600), 110.0)).font(font(14.0, 600)));
                ui.label(RichText::new("Local account").font(font(12.0, 400)).color(white(0.55)));
            });
        });
        sb.add_space(6.0);
        let y = sb.cursor().top();
        sb.painter().hline(side.left() + 10.0..=side.right() - 10.0, y, Stroke::new(1.0_f32, white(0.1)));
        sb.add_space(10.0);
    }
    let layout = if narrow { Layout::left_to_right(Align::Center) } else { Layout::top_down(Align::Min) };
    sb.with_layout(layout, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(2.0, 2.0);
        for (i, (label, tint, icon)) in SECTIONS.iter().enumerate() {
            let active = st.current == i;
            let g = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font(14.0, if active { 600 } else { 400 }), egui::Color32::PLACEHOLDER));
            let w = if narrow { g.size().x + 50.0 } else { ui.available_width() };
            let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 36.0), Sense::click());
            if active {
                ui.painter().rect_filled(r, 8.0, WHITE);
            } else {
                theme::hover_fill(ui, &resp, r, 8, white(0.1));
            }
            let ir = Rect::from_min_size(Pos2::new(r.left() + 8.0, r.center().y - 12.0), Vec2::splat(24.0));
            ui.painter().rect(ir, CornerRadius::same(7), rgb(*tint), Stroke::new(1.0_f32, white(0.25)), StrokeKind::Inside);
            let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(ir.center(), Vec2::splat(16.0))));
            icons::line_icon(&mut child, label, icon, 16.0, WHITE, 2.0);
            let text_right = ir.right() + 10.0 + g.size().x;
            ui.painter().galley(Pos2::new(ir.right() + 10.0, r.center().y - g.size().y / 2.0), g, if active { ACCENT } else { WHITE });
            // A badge on Software Update when a new version is waiting.
            if i == UPDATE_SECTION && core.update.release().is_some() {
                let c = if narrow { Pos2::new(r.right() - 10.0, r.top() + 8.0) } else { Pos2::new(r.right() - 14.0, r.center().y) };
                let _ = text_right;
                ui.painter().circle_filled(c, 9.0, egui::Color32::from_rgb(239, 68, 68));
                ui.painter().text(c, Align2::CENTER_CENTER, "1", font(11.0, 700), WHITE);
            }
            if resp.clicked() {
                st.current = i;
                st.message.clear();
                st.confirming = false;
            }
        }
    });

    // Pane
    let mut pn = ui.new_child(UiBuilder::new().max_rect(pane));
    egui::ScrollArea::vertical().id_salt("settings-pane").auto_shrink([false, false]).show(&mut pn, |ui| {
        egui::Frame::NONE.inner_margin(if narrow { Margin::same(16) } else { Margin::symmetric(24, 20) }).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(SECTIONS[st.current].0).font(display(26.0, 700)));
            ui.add_space(8.0);
            match st.current {
                0 => wallpaper_pane(ui, st, core, &name),
                1 => streamdeck_pane(ui, st, core, now),
                2 => account_pane(ui, &name),
                UPDATE_SECTION => update_pane(ui, core),
                _ => about_pane(ui),
            }
        });
    });
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(12.0);
    ui.label(RichText::new(text).font(font(13.0, 600)).color(white(0.6)));
    ui.add_space(2.0);
}
fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).font(font(12.0, 400)).color(white(0.5))).wrap());
}

fn wallpaper_pane(ui: &mut Ui, st: &mut SettingsUi, core: &mut Core, name: &str) {
    let settings = core.settings.clone();
    let using_image = settings.using_image();

    // Preview
    let w = ui.available_width().min(360.0);
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, w * 10.0 / 16.0), Sense::hover());
    let p = ui.painter().clone();
    theme::shadow(&p, r, 16, 10, 30, 0.3);
    if using_image && wallpaper::paint_image_cover_rounded(ui, &p, r, settings.custom_image.as_deref().unwrap_or_default(), 16) {
    } else {
        wallpaper::paint_preset_rounded(&p, r, settings.preset(), true, 16.0);
    }
    p.rect_stroke(r, CornerRadius::same(16), Stroke::new(1.0_f32, white(0.25)), StrokeKind::Inside);
    let label = if using_image { "Your picture" } else { settings.preset().name };
    let g = ui.fonts(|f| f.layout_no_wrap(label.into(), font(12.0, 400), WHITE));
    let lr = Rect::from_min_size(Pos2::new(r.left() + 10.0, r.bottom() - 10.0 - g.size().y - 6.0), g.size() + Vec2::new(20.0, 6.0));
    p.rect_filled(lr, CornerRadius::same(255), rgba(10, 20, 50, 0.7));
    p.galley(lr.center() - g.size() / 2.0, g, WHITE);

    heading(ui, "Built-in");
    let mut chosen = None;
    thumb_grid(ui, WALLPAPERS.len(), |ui, i, r| {
        let wp = &WALLPAPERS[i];
        let selected = !using_image && settings.preset().id == wp.id;
        let resp = ui.interact(r, egui::Id::new(("wp", i)), Sense::click());
        thumb(ui, r, wp.name, selected, resp.hovered(), |ui, ir| wallpaper::paint_preset_rounded(ui.painter(), ir, wp, true, 10.0));
        if resp.clicked() {
            chosen = Some(wp.id);
        }
    });
    if let Some(id) = chosen {
        let mut s = settings.clone();
        s.wallpaper = crate::settings::Wallpaper { kind: "preset".into(), id: Some(id.into()) };
        core.save_settings(s);
    }

    heading(ui, "Your picture");
    if let Some(path) = settings.custom_image.clone() {
        let mut use_it = false;
        thumb_grid(ui, 1, |ui, _, r| {
            let resp = ui.interact(r, egui::Id::new("wp-own"), Sense::click());
            thumb(ui, r, "Your picture", using_image, resp.hovered(), |ui, ir| {
                if !wallpaper::paint_image_cover_rounded(ui, ui.painter(), ir, &path, 10) {
                    ui.painter().rect_filled(ir, 10.0, white(0.1));
                }
            });
            use_it = resp.clicked();
        });
        if use_it {
            let mut s = settings.clone();
            s.wallpaper.kind = "image".into();
            core.save_settings(s);
        }
    }
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        let label = if settings.custom_image.is_some() { "Choose another…" } else { "Choose a picture…" };
        if theme::button(ui, label, Btn::Glass, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
            st.message.clear();
            let file = rfd::FileDialog::new().set_title("Choose a picture").add_filter("Pictures", &["png", "jpg", "jpeg", "gif", "bmp", "webp"]).pick_file();
            if let Some(file) = file {
                match import_wallpaper(name, &file) {
                    Ok(path) => {
                        let mut s = settings.clone();
                        if let Some(old) = &s.custom_image {
                            let _ = std::fs::remove_file(old);
                        }
                        s.custom_image = Some(path);
                        s.wallpaper.kind = "image".into();
                        if !core.save_settings(s) {
                            st.message = "That picture couldn't be saved on this PC.".into();
                        }
                    }
                    Err(e) => st.message = e,
                }
            }
        }
        if settings.custom_image.is_some() && theme::button(ui, "Remove picture", Btn::Danger, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
            let mut s = settings.clone();
            if let Some(old) = s.custom_image.take() {
                let _ = std::fs::remove_file(old);
            }
            s.wallpaper.kind = "preset".into();
            core.save_settings(s);
        }
    });
    if !st.message.is_empty() {
        ui.label(RichText::new(&st.message).font(font(13.0, 400)).color(RED_TEXT));
    }
    ui.add_space(4.0);
    note(ui, &format!("Saved for {name} on this PC. It will move to the database when that's connected."));
}

fn thumb_grid(ui: &mut Ui, count: usize, mut cell: impl FnMut(&mut Ui, usize, Rect)) {
    let gap = 12.0;
    let avail = ui.available_width();
    let cols = ((avail + gap) / (104.0 + gap)).floor().max(1.0) as usize;
    let w = (avail - gap * (cols as f32 - 1.0)) / cols as f32;
    let h = w * 10.0 / 16.0 + 6.0 + 16.0;
    let rows = count.div_ceil(cols);
    let (area, _) = ui.allocate_exact_size(Vec2::new(avail, rows as f32 * h + (rows.saturating_sub(1)) as f32 * gap), Sense::hover());
    for i in 0..count {
        let (row, col) = (i / cols, i % cols);
        let r = Rect::from_min_size(area.min + Vec2::new(col as f32 * (w + gap), row as f32 * (h + gap)), Vec2::new(w, h));
        cell(ui, i, r);
    }
}

fn thumb(ui: &mut Ui, r: Rect, label: &str, selected: bool, hovered: bool, paint: impl FnOnce(&mut Ui, Rect)) {
    let lift = if hovered { 1.0 } else { 0.0 };
    let ir = Rect::from_min_size(r.min - Vec2::new(0.0, lift), Vec2::new(r.width(), r.width() * 10.0 / 16.0));
    paint(ui, ir);
    let p = ui.painter();
    if selected {
        p.rect_stroke(ir, CornerRadius::same(10), Stroke::new(2.0_f32, WHITE), StrokeKind::Inside);
    } else {
        p.rect_stroke(ir, CornerRadius::same(10), Stroke::new(1.0_f32, white(0.2)), StrokeKind::Outside);
    }
    p.text(Pos2::new(r.center().x, ir.bottom() + 6.0), Align2::CENTER_TOP, label, font(12.0, if selected { 600 } else { 400 }), if selected { WHITE } else { white(0.75) });
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

fn copy_button(ui: &mut Ui, st: &mut SettingsUi, text: &str, label: &str, now: f64, small: bool) {
    let copied = st.copied.as_ref().is_some_and(|(t, at)| t == text && now - at < 1.5);
    let pad = if small { Vec2::new(14.0, 6.0) } else { Vec2::new(18.0, 8.0) };
    if theme::button(ui, if copied { "Copied" } else { label }, Btn::Glass, 13.0, None, pad).clicked() {
        ui.ctx().copy_text(text.to_string());
        st.copied = Some((text.to_string(), now));
    }
    if copied {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(300));
    }
}

fn streamdeck_pane(ui: &mut Ui, st: &mut SettingsUi, core: &mut Core, now: f64) {
    let key = core.settings.api_key.clone().unwrap_or_default();
    ui.add(egui::Label::new(RichText::new("Each Stream Deck key calls a web address. Every address ends with your personal API key, so the keys only control your own Field Guide and Time Clock.").font(font(14.0, 400)).color(white(0.75))).wrap());

    heading(ui, "Your API key");
    let connected = core.server == ServerStatus::Running;
    egui::Frame::NONE.fill(white(0.1)).stroke(Stroke::new(1.0_f32, white(0.18))).corner_radius(CornerRadius::same(14)).inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            let shown = if st.show_key { key.clone() } else { format!("{}{}", &key[..key.len().min(6)], "•".repeat(18)) };
            ui.label(RichText::new(shown).font(mono(12.5)).color(WHITE));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let text = if connected { "Connected" } else { "Not connected" };
                ui.label(RichText::new(text).font(font(12.0, 400)));
                let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                theme::dot(ui.painter(), r.center(), connected, false);
            });
        });
    });
    if let ServerStatus::Failed(msg) = &core.server {
        ui.label(RichText::new(msg).font(font(13.0, 400)).color(RED_TEXT));
    }
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        if theme::button(ui, if st.show_key { "Hide" } else { "Show" }, Btn::Glass, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
            st.show_key = !st.show_key;
        }
        copy_button(ui, st, &key, "Copy key", now, false);
        if st.confirming {
            if theme::button(ui, "Yes, replace it", Btn::Danger, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
                let mut s = core.settings.clone();
                s.api_key = Some(new_api_key());
                core.save_settings(s);
                st.confirming = false;
            }
            if theme::button(ui, "Cancel", Btn::Glass, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
                st.confirming = false;
            }
        } else if theme::button(ui, "New key…", Btn::Danger, 13.0, None, Vec2::new(18.0, 8.0)).clicked() {
            st.confirming = true;
        }
    });
    if st.confirming {
        ui.label(RichText::new("A new key stops every Stream Deck key set up with the old one.").font(font(13.0, 400)).color(RED_TEXT));
    }
    note(ui, "Keep this key to yourself. Anyone who has it can control your Field Guide.");

    heading(ui, "Addresses for your keys");
    note(ui, "In the Stream Deck app, add a \"Website\" action, paste an address, and tick \"GET request in background\". Sending a key opens the Field Guide if it's closed. Victoria Ambulance has to be open for the keys to work.");
    ui.add_space(6.0);

    let mut commands: Vec<(String, String)> = vec![
        ("Clock in / clock out (one key does both)".into(), "timeclock/toggle".into()),
        ("Tick the next step".into(), "fieldguide/step".into()),
        ("Un-tick the last step".into(), "fieldguide/unstep".into()),
        ("Start this page again".into(), "fieldguide/reset".into()),
        ("Next option (pages that ask which situation applies)".into(), "fieldguide/option".into()),
        ("Pick option 1".into(), "fieldguide/option/1".into()),
        ("Pick option 2".into(), "fieldguide/option/2".into()),
        ("Pick option 3".into(), "fieldguide/option/3".into()),
        ("MICA available: yes".into(), "fieldguide/mica/yes".into()),
        ("MICA available: no".into(), "fieldguide/mica/no".into()),
        ("Next page".into(), "fieldguide/next".into()),
        ("Previous page".into(), "fieldguide/prev".into()),
        ("Home".into(), "fieldguide/home".into()),
    ];
    for p in guide().pages.iter().filter(|p| p.id != "home") {
        commands.push((format!("Open: {}", p.title), format!("fieldguide/open/{}", p.id)));
    }
    egui::Frame::NONE.fill(white(0.1)).stroke(Stroke::new(1.0_f32, white(0.18))).corner_radius(CornerRadius::same(14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        for (i, (label, path)) in commands.iter().enumerate() {
            if i > 0 {
                let y = ui.cursor().top();
                ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0_f32, white(0.1)));
            }
            egui::Frame::NONE.inner_margin(Margin::symmetric(14, 10)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let bw = 130.0;
                    ui.allocate_ui_with_layout(Vec2::new((ui.available_width() - bw).max(80.0), 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(egui::Label::new(RichText::new(label).font(font(14.0, 500))).wrap());
                        ui.add(egui::Label::new(RichText::new(format!("/api/{path}")).font(mono(12.5)).color(white(0.55))).wrap());
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let url = format!("http://localhost:{PORT}/api/{path}?key={key}");
                        copy_button(ui, st, &url, "Copy address", now, true);
                    });
                });
            });
        }
    });
}

fn account_pane(ui: &mut Ui, name: &str) {
    let rows = [("Name", name.to_string()), ("Account type", "Local".into()), ("Stored", "This PC only".into())];
    egui::Frame::NONE.fill(white(0.1)).stroke(Stroke::new(1.0_f32, white(0.18))).corner_radius(CornerRadius::same(14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        for (i, (k, v)) in rows.iter().enumerate() {
            if i > 0 {
                let y = ui.cursor().top();
                ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0_f32, white(0.18)));
            }
            egui::Frame::NONE.inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*k).font(font(14.0, 400)).color(white(0.65)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(v).font(font(14.0, 600)));
                    });
                });
            });
        }
    });
    ui.add_space(6.0);
    note(ui, "Sign-in and syncing between PCs arrive with the database.");
}

fn about_pane(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        icons::logo(ui, 56.0);
        ui.add_space(4.0);
        ui.label(RichText::new("Victoria Ambulance").font(display(20.0, 800)));
        ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).font(font(13.0, 400)).color(white(0.7)));
        ui.add_space(4.0);
        ui.allocate_ui(Vec2::new(280.0, 60.0), |ui| {
            ui.add(egui::Label::new(RichText::new("A fictional roleplay service. Not affiliated with any real ambulance or emergency service.").font(font(12.0, 400)).color(white(0.5))).wrap());
        });
    });
}

fn update_pane(ui: &mut Ui, core: &mut Core) {
    use crate::core::UpdateRequest;
    use crate::update::{UpdateStatus, CURRENT, UPDATE_REPO};

    // This version
    egui::Frame::NONE.fill(white(0.1)).stroke(Stroke::new(1.0_f32, white(0.18))).corner_radius(CornerRadius::same(14)).inner_margin(Margin::symmetric(16, 14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            icons::logo(ui, 40.0);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new("Victoria Ambulance").font(font(15.0, 600)));
                ui.label(RichText::new(format!("Version {CURRENT}")).font(font(13.0, 400)).color(white(0.6)));
            });
        });
    });
    ui.add_space(8.0);

    // Status
    let status = core.update.clone();
    egui::Frame::NONE.fill(white(0.1)).stroke(Stroke::new(1.0_f32, white(0.18))).corner_radius(CornerRadius::same(14)).inner_margin(Margin::symmetric(16, 14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 6.0;
        match &status {
            UpdateStatus::Idle | UpdateStatus::UpToDate => {
                let (title, sub) = if status == UpdateStatus::UpToDate {
                    ("Victoria Ambulance is up to date".to_string(), format!("You have the newest version ({CURRENT})."))
                } else {
                    ("Updates are checked automatically".to_string(), "The app looks for a new version when it starts and every few hours.".to_string())
                };
                status_line(ui, &title, status == UpdateStatus::UpToDate);
                ui.label(RichText::new(sub).font(font(13.0, 400)).color(white(0.65)));
            }
            UpdateStatus::Checking => {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new().size(14.0).color(WHITE));
                    ui.label(RichText::new("Checking for updates…").font(font(14.0, 600)));
                });
            }
            UpdateStatus::Available(r) | UpdateStatus::Downloading(r, _) | UpdateStatus::Ready(r, _) | UpdateStatus::Installing(r) => {
                ui.label(RichText::new(format!("Victoria Ambulance {} is ready to install", r.version)).font(font(15.0, 600)));
                ui.label(RichText::new(format!("You have version {CURRENT}. The app restarts to finish installing — you'll log in again, and the Time Clock keeps running.")).font(font(13.0, 400)).color(white(0.65)));
                if !r.notes.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new("What's new").font(font(13.0, 600)).color(white(0.6)));
                    egui::Frame::NONE.fill(rgba(0, 0, 0, 0.18)).corner_radius(CornerRadius::same(10)).inner_margin(Margin::same(10)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(egui::Label::new(RichText::new(clean_notes(&r.notes)).font(font(13.0, 400))).wrap());
                    });
                }
                ui.add_space(4.0);
                match &status {
                    UpdateStatus::Downloading(_, p) => progress(ui, *p, &format!("Downloading… {}%", (p * 100.0).round())),
                    UpdateStatus::Ready(..) | UpdateStatus::Installing(_) => progress(ui, 1.0, "Installing…"),
                    _ => {
                        if theme::button(ui, "Install now", Btn::Primary, 13.0, None, Vec2::new(20.0, 8.0)).clicked() {
                            core.update_request = Some(UpdateRequest::Install);
                        }
                    }
                }
            }
            UpdateStatus::Failed(e) => {
                ui.label(RichText::new("The update didn't work").font(font(15.0, 600)));
                ui.add(egui::Label::new(RichText::new(e).font(font(13.0, 400)).color(RED_TEXT)).wrap());
            }
        }
    });
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        let busy = core.update.busy();
        if theme::button_enabled(ui, "Check for updates", Btn::Glass, 13.0, None, Vec2::new(18.0, 8.0), !busy).clicked() {
            core.update_request = Some(UpdateRequest::Check);
        }
        if let Some(at) = core.update_checked_at {
            ui.label(RichText::new(format!("Last checked {}", crate::ui::format::time(at))).font(font(12.0, 400)).color(white(0.5)));
        }
    });
    ui.add_space(4.0);
    note(ui, &format!("Updates come from github.com/{UPDATE_REPO}."));
}

fn status_line(ui: &mut Ui, title: &str, ok: bool) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
        theme::dot(ui.painter(), r.center(), ok, ok);
        ui.label(RichText::new(title).font(font(15.0, 600)));
    });
}

fn progress(ui: &mut Ui, frac: f32, label: &str) {
    ui.label(RichText::new(label).font(font(13.0, 500)));
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().min(320.0), 6.0), Sense::hover());
    ui.painter().rect_filled(r, 3.0, white(0.18));
    ui.painter().rect_filled(Rect::from_min_size(r.min, Vec2::new(r.width() * frac.clamp(0.0, 1.0), 6.0)), 3.0, WHITE);
}

/// Release notes are written in Markdown on GitHub; show them as plain text.
pub fn clean_notes(notes: &str) -> String {
    notes
        .lines()
        .map(|l| {
            let t = l.trim_start();
            let t = t.trim_start_matches('#').trim_start();
            if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) { format!("•  {rest}") } else { t.to_string() }
        })
        .map(|l| l.replace("**", "").replace('`', ""))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
