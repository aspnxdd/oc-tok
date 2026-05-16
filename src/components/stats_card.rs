use freya::prelude::*;

#[derive(PartialEq)]
pub struct StatsCard {
    pub label: String,
    pub value: String,
    pub color: Color,
}

impl Component for StatsCard {
    fn render(&self) -> impl IntoElement {
        rect()
            .background((30, 30, 34))
            .corner_radius(CornerRadius::new_all(12.))
            .padding(Gaps::new(20., 20., 20., 20.))
            .vertical()
            .spacing(8.)
            .child(
                label()
                    .text(self.label.clone())
                    .color((160, 160, 160))
                    .font_size(12.)
                    .font_weight(FontWeight::BOLD),
            )
            .child(
                label()
                    .text(self.value.clone())
                    .color(self.color)
                    .font_size(28.)
                    .font_weight(FontWeight::BOLD),
            )
    }
}
