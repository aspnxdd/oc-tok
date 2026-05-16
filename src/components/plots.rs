use freya::prelude::*;
use freya::plot::PlotSkiaBackend;
use plotters::prelude::*;
use plotters::style::Color as PlottersColor;
use std::collections::HashMap;

use crate::data::DailyStats;

#[derive(PartialEq, Clone)]
pub struct DailyTokensPlot {
    pub daily: HashMap<chrono::NaiveDate, DailyStats>,
}

impl Component for DailyTokensPlot {
    fn render(&self) -> impl IntoElement {
        let daily = self.daily.clone();

        canvas(RenderCallback::new(move |ctx| {
            let area = ctx.layout_node.visible_area();
            let size = (area.width() as i32, area.height() as i32);
            let backend = PlotSkiaBackend::new(ctx.canvas, ctx.font_collection, size);
            let root = backend.into_drawing_area();
            let _ = root.fill(&RGBColor(30, 30, 34));

            if daily.is_empty() {
                return;
            }

            let mut dates: Vec<chrono::NaiveDate> = daily.keys().cloned().collect();
            dates.sort();

            let max_tokens = daily
                .values()
                .map(|d| d.input.max(d.output))
                .max()
                .unwrap_or(1)
                .max(1);

            let n = dates.len() as f64;
            let max_y = max_tokens as f64;

            let mut chart = match ChartBuilder::on(&root)
                .caption("Daily Tokens", ("sans-serif", 14).into_font().color(&WHITE))
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(45)
                .build_cartesian_2d(0.0f64..n, 0.0f64..max_y)
            {
                Ok(c) => c,
                Err(_) => return,
            };

            let _ = chart
                .configure_mesh()
                .x_labels(dates.len())
                .x_label_formatter(&|x| {
                    let idx = *x as usize;
                    if idx < dates.len() {
                        dates[idx].format("%m-%d").to_string()
                    } else {
                        String::new()
                    }
                })
                .label_style(("sans-serif", 9).into_font().color(&RGBColor(160, 160, 160)))
                .axis_style(&RGBColor(58, 58, 64))
                .draw();

            let bar_width = 0.35;

            for (i, date) in dates.iter().enumerate() {
                let stats = daily.get(date).unwrap();
                let x = i as f64;

                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [(x - bar_width, 0.0), (x, stats.input as f64)],
                    RGBColor(60, 130, 200).filled(),
                )));

                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [(x, 0.0), (x + bar_width, stats.output as f64)],
                    RGBColor(200, 80, 80).filled(),
                )));
            }

            let _ = root.present();
        }))
        .height(Size::fill())
        .width(Size::fill())
    }
}

#[derive(PartialEq, Clone)]
pub struct ModelBreakdownPlot {
    pub model_breakdown: HashMap<String, u64>,
}

impl Component for ModelBreakdownPlot {
    fn render(&self) -> impl IntoElement {
        let breakdown = self.model_breakdown.clone();

        canvas(RenderCallback::new(move |ctx| {
            let area = ctx.layout_node.visible_area();
            let size = (area.width() as i32, area.height() as i32);
            let backend = PlotSkiaBackend::new(ctx.canvas, ctx.font_collection, size);
            let root = backend.into_drawing_area();
            let _ = root.fill(&RGBColor(30, 30, 34));

            if breakdown.is_empty() {
                return;
            }

            let mut models: Vec<(String, u64)> = breakdown.iter().map(|(k, v)| (k.clone(), *v)).collect();
            models.sort_by(|a, b| b.1.cmp(&a.1));

            let max_tokens = models.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
            let n = models.len() as f64;

            let mut chart = match ChartBuilder::on(&root)
                .caption("Model Breakdown", ("sans-serif", 14).into_font().color(&WHITE))
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(100)
                .build_cartesian_2d(0u64..max_tokens, 0.0f64..n)
            {
                Ok(c) => c,
                Err(_) => return,
            };

            let _ = chart
                .configure_mesh()
                .y_labels(models.len())
                .y_label_formatter(&|y| {
                    let idx = *y as usize;
                    if idx < models.len() {
                        let name = &models[idx].0;
                        if name.len() > 18 {
                            format!("{}...", &name[..18])
                        } else {
                            name.clone()
                        }
                    } else {
                        String::new()
                    }
                })
                .label_style(("sans-serif", 9).into_font().color(&RGBColor(160, 160, 160)))
                .axis_style(&RGBColor(58, 58, 64))
                .draw();

            let bar_height = 0.6;
            for (i, (_model, tokens)) in models.iter().enumerate() {
                let y = i as f64 + 0.5;
                let _ = chart.draw_series(std::iter::once(Rectangle::new(
                    [(0, y - bar_height / 2.0), (*tokens, y + bar_height / 2.0)],
                    RGBColor(100, 160, 220).filled(),
                )));
            }

            let _ = root.present();
        }))
        .height(Size::fill())
        .width(Size::fill())
    }
}

#[derive(PartialEq, Clone)]
pub struct DailyCostPlot {
    pub daily: HashMap<chrono::NaiveDate, DailyStats>,
}

impl Component for DailyCostPlot {
    fn render(&self) -> impl IntoElement {
        let daily = self.daily.clone();

        canvas(RenderCallback::new(move |ctx| {
            let area = ctx.layout_node.visible_area();
            let size = (area.width() as i32, area.height() as i32);
            let backend = PlotSkiaBackend::new(ctx.canvas, ctx.font_collection, size);
            let root = backend.into_drawing_area();
            let _ = root.fill(&RGBColor(30, 30, 34));

            if daily.is_empty() {
                return;
            }

            let mut dates: Vec<chrono::NaiveDate> = daily.keys().cloned().collect();
            dates.sort();

            let max_cost = daily
                .values()
                .map(|d| d.cost)
                .fold(0.0f64, |a, b| a.max(b))
                .max(0.01);

            let n = dates.len() as f64;

            let mut chart = match ChartBuilder::on(&root)
                .caption("Daily Cost", ("sans-serif", 14).into_font().color(&WHITE))
                .margin(8)
                .x_label_area_size(30)
                .y_label_area_size(45)
                .build_cartesian_2d(0.0f64..n, 0.0f64..max_cost)
            {
                Ok(c) => c,
                Err(_) => return,
            };

            let _ = chart
                .configure_mesh()
                .x_labels(dates.len())
                .x_label_formatter(&|x| {
                    let idx = *x as usize;
                    if idx < dates.len() {
                        dates[idx].format("%m-%d").to_string()
                    } else {
                        String::new()
                    }
                })
                .label_style(("sans-serif", 9).into_font().color(&RGBColor(160, 160, 160)))
                .axis_style(&RGBColor(58, 58, 64))
                .draw();

            let points: Vec<(f64, f64)> = dates
                .iter()
                .enumerate()
                .map(|(i, date)| {
                    let stats = daily.get(date).unwrap();
                    (i as f64, stats.cost)
                })
                .collect();

            let _ = chart.draw_series(LineSeries::new(
                points.into_iter(),
                RGBColor(80, 200, 120).stroke_width(2),
            ));

            let _ = root.present();
        }))
        .height(Size::fill())
        .width(Size::fill())
    }
}
