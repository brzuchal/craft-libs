use super::*;

const TOP: &[FileNavItem<'static, &str>] =
    &[FileNavItem { key: "home", label: "Home" }, FileNavItem { key: "new", label: "New" }, FileNavItem { key: "open", label: "Open" }];
const REST: &[FileNavItem<'static, &str>] = &[FileNavItem { key: "info", label: "Info" }, FileNavItem { key: "print", label: "Print" }];
const FOOT: &[FileNavItem<'static, &str>] = &[
    FileNavItem { key: "options", label: "Options" },
    FileNavItem { key: "community", label: "A community with a name far too long for the column" },
    FileNavItem { key: "about", label: "About" },
];

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx
}

fn frame(ctx: &egui::Context, band: f32, height: f32, events: Vec<egui::Event>) -> (Option<FileNavAction<&'static str>>, egui::FullOutput) {
    let style = FileNavStyle::on_accent(Color32::from_rgb(0x3B, 0x5B, 0xDB));
    let input = egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(600.0, height))), events, ..Default::default() };
    let mut action = None;
    let mut full = ctx.run_ui(input, |ui| {
        let nothing = |_: Option<&str>, _: &Painter, _: Rect, _: Color32| {};
        action = FileNav::new("Back", &[TOP, REST], Some("open"), &style, &nothing).foot(FOOT).title_band(band).show(ui).action;
    });
    full.textures_delta.clear();
    (action, full)
}

fn node<'a>(full: &'a egui::FullOutput, label: &str) -> Option<&'a egui::accesskit::Node> {
    let update = full.platform_output.accesskit_update.as_ref().expect("accesskit is on");
    update.nodes.iter().map(|(_, n)| n).find(|n| n.label() == Some(label))
}

fn centre(full: &egui::FullOutput, label: &str) -> egui::Pos2 {
    let b = node(full, label).expect("node with the label").bounds().expect("bounds");
    pos2(((b.x0 + b.x1) / 2.0) as f32, ((b.y0 + b.y1) / 2.0) as f32)
}

fn click(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE }
}

fn press(label: &str) -> Option<FileNavAction<&'static str>> {
    let ctx = context();
    let p = centre(&frame(&ctx, 0.0, 700.0, vec![]).1, label);
    frame(&ctx, 0.0, 700.0, vec![egui::Event::PointerMoved(p), click(p, true)]);
    frame(&ctx, 0.0, 700.0, vec![click(p, false)]).0
}

#[test]
fn back_pages_and_the_foot_report_what_was_chosen() {
    assert_eq!(press("Back"), Some(FileNavAction::Back));
    assert_eq!(press("New"), Some(FileNavAction::Choose("new")));
    assert_eq!(press("Print"), Some(FileNavAction::Choose("print")), "a page of the second group");
    assert_eq!(press("Options"), Some(FileNavAction::Choose("options")), "an entry of the foot");
    assert_eq!(frame(&context(), 0.0, 700.0, vec![]).0, None, "nothing without input");
}

#[test]
fn the_page_that_is_showing_is_the_only_one_announced_as_chosen() {
    let (_, full) = frame(&context(), 0.0, 700.0, vec![]);
    let toggled = |label: &str| node(&full, label).expect("entry").toggled();
    assert_eq!(toggled("Open"), Some(egui::accesskit::Toggled::True));
    assert_eq!(toggled("Home"), Some(egui::accesskit::Toggled::False));
    assert_eq!(toggled("Back"), None, "Back is an action, not a page");
}

#[test]
fn the_order_is_back_then_the_groups_then_the_foot_at_the_bottom() {
    let (_, full) = frame(&context(), 0.0, 700.0, vec![]);
    let y = |label: &str| centre(&full, label).y;
    assert!(y("Back") < y("Home") && y("Open") < y("Info") && y("Print") < y("Options") && y("Options") < y("About"));
    assert!(y("Info") - y("Open") > y("Open") - y("New"), "a rule separates the groups");
    assert!(y("About") > 700.0 - 60.0, "the foot sits at the bottom of the column");
}

#[test]
fn a_title_band_keeps_back_clear_of_the_window_buttons_and_drags_the_window() {
    let ctx = context();
    let without = centre(&frame(&ctx, 0.0, 700.0, vec![]).1, "Back").y;
    let with = centre(&frame(&ctx, 28.0, 700.0, vec![]).1, "Back").y;
    assert!(with > 28.0 && with > without, "Back sits under the band, at {with}");

    let ctx = context();
    let (from, to) = (pos2(100.0, 14.0), pos2(140.0, 20.0));
    frame(&ctx, 28.0, 700.0, vec![]);
    frame(&ctx, 28.0, 700.0, vec![egui::Event::PointerMoved(from), click(from, true)]);
    assert_eq!(frame(&ctx, 28.0, 700.0, vec![egui::Event::PointerMoved(to)]).0, Some(FileNavAction::DragWindow));
}

#[test]
fn a_label_longer_than_the_column_stays_inside_it() {
    let (_, full) = frame(&context(), 0.0, 700.0, vec![]);
    let b = node(&full, "A community with a name far too long for the column").expect("the long entry").bounds().expect("bounds");
    assert!(b.x1 <= 200.0, "the row is as wide as the column, not as the label: {}", b.x1);
}

#[test]
fn a_column_too_short_for_its_entries_scrolls_instead_of_overlapping() {
    for height in [1.0, 40.0, 200.0] {
        assert_eq!(frame(&context(), 28.0, height, vec![]).0, None);
    }
    // 300 is too short for Back, five pages and three foot entries: the foot follows the pages.
    let (_, full) = frame(&context(), 0.0, 300.0, vec![]);
    let y = |label: &str| centre(&full, label).y;
    assert!(y("Options") > y("Print") + 20.0, "the foot is under the last page, not over it");
    assert!(y("About") > 300.0, "and the column scrolls to reach the rest");
}

#[test]
fn the_neutral_style_marks_the_page_with_the_accent() {
    let v = egui::Visuals::light();
    let (neutral, filled) = (FileNavStyle::from_visuals(&v), FileNavStyle::on_accent(v.selection.bg_fill));
    assert_eq!(neutral.mark, v.selection.bg_fill);
    assert_eq!(filled.fill, v.selection.bg_fill);
    assert_ne!(neutral.fill, filled.fill);
    assert_eq!(neutral.width, filled.width, "only the colours differ");
}
