use freya::prelude::*;

#[derive(PartialEq)]
pub struct StatsCard {
    pub label: String,
    pub value: String,
    pub color: Color,
}

impl Component for StatsCard {
    fn render(&self) -> impl IntoElement {
        let label_color = use_theme().read().colors.text_secondary;

        Card::new().width(Size::flex(1.)).child(
            rect()
                .vertical()
                .spacing(8.)
                .child(
                    label()
                        .text(self.label.clone())
                        .color(label_color)
                        .font_size(12.)
                        .font_weight(FontWeight::BOLD),
                )
                .child(
                    label()
                        .text(self.value.clone())
                        .color(self.color)
                        .font_size(28.)
                        .font_weight(FontWeight::BOLD),
                ),
        )
    }
}
