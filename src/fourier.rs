use eframe::epaint::{Color32, Shape};
use egui::{Painter, Pos2, Stroke, Ui};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;

pub struct Fourier {
    data: FourierData,
    time: f32,
    path: Vec<(f32, f32)>,
}

#[derive(Serialize, Deserialize)]
struct FourierData {
    epicycles: Vec<FourierEpicycle>,
    metadata: FourierMetadata,
}

#[derive(Serialize, Deserialize)]
struct FourierMetadata {
    height: f32,
    width: f32,
}

#[derive(Serialize, Deserialize)]
struct FourierEpicycle {
    re: f32,
    im: f32,
    freq: f32,
    amp: f32,
    phase: f32,
}

impl Fourier {
    pub fn from_json_file(file: String) -> anyhow::Result<Fourier> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        let mut fourier: FourierData = serde_json::from_reader(reader)?;
        fourier.epicycles.sort_by(|a, b| b.amp.total_cmp(&a.amp));

        Ok(Fourier {
            data: fourier,
            time: 0.0,
            path: vec![],
        })
    }

    pub fn ui(&mut self, ui: &mut Ui, left_offset: f32, top_offset: f32) {
        let left_offset = left_offset + ui.available_rect_before_wrap().width() / 2.;
        let top_offset = top_offset + ui.available_rect_before_wrap().height() / 2.;

        let img_scalar = Self::img_scalars(
            (&self.data.metadata.width, &self.data.metadata.height),
            (
                &ui.available_rect_before_wrap().width(),
                &ui.available_rect_before_wrap().height(),
            ),
        );

        ui.ctx().request_repaint();
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        let ((x, y), epicycles) =
            self.epi_cycles(left_offset, top_offset, 0.0, img_scalar, self.time);
        self.time += PI * 2. / self.data.epicycles.len() as f32;
        self.path.push((
            (x - left_offset) / img_scalar,
            (y - top_offset) / img_scalar,
        ));

        let mut path_shapes = vec![];
        for ((x, y), (a, b)) in self
            .path
            .iter()
            .map(|(x, y)| (x * img_scalar + left_offset, y * img_scalar + top_offset))
            .tuple_windows()
        {
            path_shapes.push(Shape::line_segment(
                [Pos2::new(x, y), Pos2::new(a, b)],
                Stroke::new(1.0, Color32::WHITE),
            ));
        }
        painter.extend(path_shapes);
        painter.extend(epicycles);

        ui.expand_to_include_rect(painter.clip_rect());

        if self.time > PI * 2. {
            self.time = 0.0;
            self.path.clear();
        }
    }

    fn epi_cycles(
        &mut self,
        mut x: f32,
        mut y: f32,
        rotation: f32,
        scale: f32,
        time: f32,
    ) -> ((f32, f32), Vec<Shape>) {
        let mut shapes = vec![];
        let mut prevx = x;
        let mut prevy = y;
        for epicycle in &self.data.epicycles {
            let freq = epicycle.freq;
            let radius = epicycle.amp * scale;
            let phase = epicycle.phase;
            x += radius * (rotation + freq * time + phase).cos();
            y += radius * (rotation + freq * time + phase).sin();
            shapes.push(Shape::line_segment(
                [Pos2::new(prevx, prevy), Pos2::new(x, y)],
                Stroke::new(0.01, Color32::WHITE),
            ));
            shapes.push(Shape::circle_stroke(
                Pos2::new(prevx, prevy),
                radius,
                Stroke::new(0.01, Color32::WHITE),
            ));
            prevx = x;
            prevy = y;
        }
        ((x, y), shapes)
    }

    fn img_scalars((ori_x, ori_y): (&f32, &f32), (width, height): (&f32, &f32)) -> f32 {
        if width * ori_y > height * ori_x {
            height / ori_y
        } else {
            width / ori_x
        }
    }
}
