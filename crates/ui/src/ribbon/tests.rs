use super::*;

fn frame(ctx: &egui::Context, frame: RibbonFrame, captions: bool) -> (Rect, Vec<Rect>) {
    let style = RibbonStyle::from_visuals(&egui::Visuals::light());
    let mut outer = Rect::NOTHING;
    let mut groups = Vec::new();
    let input = egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 200.0))), ..Default::default() };
    let mut full = ctx.run_ui(input, |ui| {
        let r = Ribbon::new(&style).frame(frame).captions(captions).show(ui, |ui| {
            for caption in ["Clipboard", "Font", "Paragraph"] {
                let before = ui.cursor().min.x;
                RibbonGroup::new(caption, &style).launcher("More").show(ui, |ui| {
                    ui.allocate_exact_size(vec2(40.0, 20.0), Sense::hover());
                });
                groups.push(Rect::from_min_max(egui::pos2(before, 0.0), egui::pos2(ui.cursor().min.x, 0.0)));
            }
        });
        outer = r.response.rect;
    });
    full.textures_delta.clear();
    (outer, groups)
}

#[test]
fn captions_add_their_row_and_a_card_adds_its_margin() {
    let ctx = egui::Context::default();
    let style = RibbonStyle::from_visuals(&egui::Visuals::light());
    let (with, _) = frame(&ctx, RibbonFrame::Card { margin: 4.0 }, true);
    let (without, _) = frame(&ctx, RibbonFrame::Card { margin: 4.0 }, false);
    let (flush, _) = frame(&ctx, RibbonFrame::Flush, false);
    assert_eq!(with.height() - without.height(), CAPTION_H);
    assert_eq!(without.height(), style.content_height + PAD_Y * 2.0 + 4.0);
    assert_eq!(flush.height(), style.content_height + PAD_Y * 2.0 + 1.0);
}

#[test]
fn groups_follow_each_other_and_are_at_least_as_wide_as_their_caption() {
    let ctx = egui::Context::default();
    let (_, groups) = frame(&ctx, RibbonFrame::Flush, true);
    assert_eq!(groups.len(), 3);
    for pair in groups.windows(2) {
        assert!((pair[0].right() - pair[1].left()).abs() < 0.5, "groups are contiguous: {pair:?}");
    }
    // "Paragraph" plus the launcher is wider than the 40 px of content.
    assert!(groups[2].width() > 40.0 + GROUP_PAD * 2.0);
}
