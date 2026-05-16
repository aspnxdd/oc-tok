use chrono::NaiveDate;
use freya::plot::PlotSkiaBackend;
use freya::prelude::{Color as FreyaColor, *};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::Color as PlottersColor;

use crate::data::DailyStats;

type PlotArea<'a> = DrawingArea<PlotSkiaBackend<'a>, Shift>;

fn date_label(daily: &[(NaiveDate, DailyStats)], x: f64) -> String {
    daily
        .get(x as usize)
        .map_or(String::new(), |(date, _)| date.format("%m-%d").to_string())
}

#[derive(Clone, Copy)]
struct PlotPalette {
    fill: RGBColor,
    text: RGBColor,
    axis: RGBColor,
    input: RGBColor,
    output: RGBColor,
    accent: RGBColor,
    cost: RGBColor,
}

impl PlotPalette {
    fn from_theme() -> Self {
        let colors = &use_theme().read().colors;
        Self {
            fill: rgb(colors.surface_secondary),
            text: rgb(colors.text_secondary),
            axis: rgb(colors.border),
            input: rgb(colors.info),
            output: rgb(colors.error),
            accent: rgb(colors.info),
            cost: rgb(colors.success),
        }
    }
}

fn rgb(color: FreyaColor) -> RGBColor {
    RGBColor(color.r(), color.g(), color.b())
}

fn plot_canvas<F>(fill: RGBColor, draw: F) -> impl IntoElement
where
    F: Fn(PlotArea<'_>) + 'static,
{
    canvas(RenderCallback::new(move |ctx| {
        let area = ctx.layout_node.visible_area();
        let size = (area.width() as i32, area.height() as i32);
        let backend = PlotSkiaBackend::new(ctx.canvas, ctx.font_collection, size);
        let root = backend.into_drawing_area();
        let _ = root.fill(&fill);
        draw(root);
    }))
    .expanded()
}

#[derive(PartialEq, Clone)]
pub struct DailyTokensPlot {
    pub daily: Vec<(NaiveDate, DailyStats)>,
}

impl Component for DailyTokensPlot {
    fn render(&self) -> impl IntoElement {
        let palette = PlotPalette::from_theme();
        let daily = self.daily.clone();

        plot_canvas(palette.fill, move |root| {
            if daily.is_empty() {
                return;
            }
            let max_tokens = daily
                .iter()
                .map(|(_, day)| day.input.max(day.output))
                .max()
                .unwrap_or(1)
                .max(1);

            let count = daily.len() as f64;
            let max_y = max_tokens as f64;

            let Ok(mut chart) = ChartBuilder::on(&root)
                .caption("Daily Tokens", ("sans-serif", 14).into_font().color(&WHITE))
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(45)
                .build_cartesian_2d(0.0f64..count, 0.0f64..max_y)
            else {
                return;
            };

            let _ = chart
                .configure_mesh()
                .x_labels(daily.len())
                .x_label_formatter(&|x| date_label(&daily, *x))
                .label_style(("sans-serif", 9).into_font().color(&palette.text))
                .axis_style(&palette.axis)
                .draw();

            let bar_width = 0.35;
            for (index, (_, stats)) in daily.iter().enumerate() {
                let x_center = index as f64;
                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_center - bar_width, 0.0), (x_center, stats.input as f64)],
                    palette.input.filled(),
                )));
                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_center, 0.0), (x_center + bar_width, stats.output as f64)],
                    palette.output.filled(),
                )));
            }
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct ModelBreakdownPlot {
    pub models: Vec<(String, u64)>,
}

impl Component for ModelBreakdownPlot {
    fn render(&self) -> impl IntoElement {
        let palette = PlotPalette::from_theme();
        let models = self.models.clone();

        plot_canvas(palette.fill, move |root| {
            if models.is_empty() {
                return;
            }

            let max_tokens = models
                .iter()
                .map(|(_, tokens)| *tokens)
                .max()
                .unwrap_or(1)
                .max(1);
            let count = models.len();

            let Ok(mut chart) = ChartBuilder::on(&root)
                .caption(
                    "Model Breakdown",
                    ("sans-serif", 14).into_font().color(&WHITE),
                )
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(140)
                .build_cartesian_2d(0u64..max_tokens, -0.5_f64..(count as f64 - 0.5))
            else {
                return;
            };

            let _ = chart
                .configure_mesh()
                .y_labels(count)
                .y_label_formatter(&|y| {
                    let idx = y.round() as usize;
                    let Some((name, _)) = models.get(idx) else {
                        return String::new();
                    };
                    if name.chars().count() > 22 {
                        format!("{}...", name.chars().take(22).collect::<String>())
                    } else {
                        name.clone()
                    }
                })
                .label_style(("sans-serif", 9).into_font().color(&palette.text))
                .axis_style(&palette.axis)
                .draw();

            let bar_height = 0.6;
            for (index, (_, tokens)) in models.iter().enumerate() {
                let y_center = index as f64;
                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [
                        (0, y_center - bar_height / 2.0),
                        (*tokens, y_center + bar_height / 2.0),
                    ],
                    palette.accent.filled(),
                )));
            }
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct DailyCostPlot {
    pub daily: Vec<(NaiveDate, DailyStats)>,
}

impl Component for DailyCostPlot {
    fn render(&self) -> impl IntoElement {
        let palette = PlotPalette::from_theme();
        let daily = self.daily.clone();

        plot_canvas(palette.fill, move |root| {
            if daily.is_empty() {
                return;
            }

            let max_cost = daily
                .iter()
                .map(|(_, day)| day.cost)
                .fold(0.0f64, f64::max)
                .max(0.01);
            let count = daily.len() as f64;

            let Ok(mut chart) = ChartBuilder::on(&root)
                .caption("Daily Cost", ("sans-serif", 14).into_font().color(&WHITE))
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(45)
                .build_cartesian_2d(0.0f64..count, 0.0f64..max_cost)
            else {
                return;
            };

            let _ = chart
                .configure_mesh()
                .x_labels(daily.len())
                .x_label_formatter(&|x| date_label(&daily, *x))
                .label_style(("sans-serif", 9).into_font().color(&palette.text))
                .axis_style(&palette.axis)
                .draw();

            let points = daily
                .iter()
                .enumerate()
                .map(|(index, (_, day))| (index as f64, day.cost));
            let _ = chart.draw_series(LineSeries::new(points, palette.cost.stroke_width(2)));
        })
    }
}
