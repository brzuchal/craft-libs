use super::*;
use egui::{Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput};

struct SliderTest {
    ctx: Context,
    style: SliderStyle,
    value: f64,
    log: bool,
    step: Option<f64>,
    field: bool,
    rect: Option<Rect>,
    changes: usize,
}

impl SliderTest {
    fn new(value: f64) -> Self {
        let ctx = Context::default();
        ctx.enable_accesskit();
        ctx.global_style_mut(|s| s.animation_time = 0.0);
        let style = SliderStyle::from_visuals(&egui::Visuals::light());
        Self { ctx, style, value, log: false, step: None, field: false, rect: None, changes: 0 }
    }

    fn frame(&mut self, events: Vec<Event>) -> (Response, FullOutput) {
        let ctx = self.ctx.clone();
        let mut response = None;
        let mut output =
            ctx.run_ui(RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 100.0))), events, ..Default::default() }, |ui| {
                let mut s = Slider::new(&mut self.value, 10.0..=400.0, "Zoom", &self.style).width(200.0).mark(100.0);
                if self.log {
                    s = s.scale(Scale::Logarithmic);
                }
                if let Some(step) = self.step {
                    s = s.step(step);
                }
                if self.field {
                    s = s.editable(0, "%");
                }
                let r = s.show(ui);
                self.changes += usize::from(r.changed());
                self.rect = Some(r.rect);
                response = Some(r);
            });
        output.textures_delta.clear();
        (response.unwrap(), output)
    }

    /// The track's left and right thumb centres, from the first frame's layout.
    fn track(&mut self) -> (f32, f32, f32) {
        let (r, _) = self.frame(vec![]);
        (r.rect.left() + THUMB_RADIUS, r.rect.right() - THUMB_RADIUS, r.rect.center().y)
    }

    fn click(&mut self, x: f32, y: f32) {
        let p = pos2(x, y);
        self.frame(vec![Event::PointerMoved(p), pointer(p, true)]);
        self.frame(vec![pointer(p, false)]);
        self.frame(vec![]);
    }
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
}

fn key(key: Key) -> Event {
    Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }
}

#[test]
fn clicking_the_track_sets_the_value_linearly_and_logarithmically() {
    let mut t = SliderTest::new(100.0);
    let (x0, x1, y) = t.track();
    t.click(x0 + (x1 - x0) * 0.5, y);
    assert!((t.value - 205.0).abs() < 1.0, "linear midpoint of 10–400: {}", t.value);
    assert!(t.changes > 0);

    let mut t = SliderTest::new(100.0);
    t.log = true;
    t.step = Some(1.0);
    let (x0, x1, y) = t.track();
    t.click(x0 + (x1 - x0) * 0.5, y);
    assert_eq!(t.value, 63.0, "logarithmic midpoint of 10–400 is √4000 ≈ 63, snapped to whole percent");
    t.click(x0, y);
    assert_eq!(t.value, 10.0);
    t.click(x1 + 50.0, y);
    assert_eq!(t.value, 10.0, "a click past the track isn't on the slider");
}

#[test]
fn double_click_returns_to_the_mark() {
    let mut t = SliderTest::new(250.0);
    t.log = true;
    let (x0, _, y) = t.track();
    let p = pos2(x0 + 20.0, y);
    t.frame(vec![Event::PointerMoved(p), pointer(p, true)]);
    t.frame(vec![pointer(p, false)]);
    t.frame(vec![pointer(p, true)]);
    t.frame(vec![pointer(p, false)]);
    t.frame(vec![]);
    assert_eq!(t.value, 100.0);
}

#[test]
fn keyboard_moves_by_step_page_and_to_the_ends() {
    let mut t = SliderTest::new(100.0);
    t.step = Some(1.0);
    let (r, _) = t.frame(vec![]);
    t.ctx.memory_mut(|m| m.request_focus(r.id));
    t.frame(vec![]);
    t.frame(vec![key(Key::ArrowRight)]);
    assert_eq!(t.value, 101.0);
    t.frame(vec![key(Key::PageDown)]);
    assert_eq!(t.value, 91.0);
    t.frame(vec![key(Key::End)]);
    assert_eq!(t.value, 400.0);
    t.frame(vec![key(Key::Home)]);
    assert_eq!(t.value, 10.0);
}

#[test]
fn the_field_keeps_its_width_and_the_track_does_not_move() {
    let mut t = SliderTest::new(99.0);
    t.field = true;
    let (a, _) = t.frame(vec![]);
    t.value = 100.0;
    let (b, _) = t.frame(vec![]);
    assert_eq!(a.rect, b.rect, "track and field together keep their size from 99% to 100%");
}

#[test]
fn non_finite_values_and_empty_ranges_are_harmless() {
    let mut t = SliderTest::new(f64::NAN);
    t.frame(vec![]);
    assert_eq!(t.value, 10.0);
    let style = SliderStyle::from_visuals(&egui::Visuals::dark());
    let ctx = Context::default();
    let mut v = 5.0;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        Slider::new(&mut v, 3.0..=3.0, "Empty", &style).scale(Scale::Logarithmic).show(ui);
    });
    out.textures_delta.clear();
    assert_eq!(v, 5.0, "showing never changes a value nobody touched");
}

#[test]
fn assistive_technology_sees_a_labelled_slider() {
    let mut t = SliderTest::new(100.0);
    let (_, out) = t.frame(vec![]);
    let update = out.platform_output.accesskit_update.expect("accesskit is on");
    let node = update.nodes.iter().map(|(_, n)| n).find(|n| n.role() == egui::accesskit::Role::Slider).expect("a slider node");
    assert_eq!(node.numeric_value(), Some(100.0));
    assert_eq!(node.label(), Some("Zoom"));
}

#[test]
fn the_field_stays_beside_the_track_in_a_vertical_layout() {
    let ctx = Context::default();
    let style = SliderStyle::from_visuals(&egui::Visuals::light());
    let mut v = 1.0;
    let mut rect = None;
    let mut out = ctx.run_ui(RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))), ..Default::default() }, |ui| {
        ui.vertical(|ui| rect = Some(Slider::new(&mut v, 0.5..=2.0, "Speed", &style).editable(2, "×").show(ui).rect));
    });
    out.textures_delta.clear();
    assert!(rect.unwrap().height() <= HEIGHT + 4.0, "one row: {:?}", rect);
}
