use geo::kernels::HasKernel;
use geo_types::{CoordFloat, Rect};
use std::path::Path;

use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_line_segment_mut, Canvas};

use crate::rtree::{Index, RTree};

pub struct TreeRenderOptions {
    width: u32,
    height: u32,
    threshold: Option<usize>,
}

impl TreeRenderOptions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            threshold: None,
        }
    }

    pub fn with_threshold(&mut self, threshold: usize) -> &mut Self {
        self.threshold = Some(threshold);

        self
    }

    pub fn without_threshold(&mut self) -> &mut Self {
        self.threshold = None;

        self
    }

    pub fn draw_tree<P: AsRef<Path>, ND, T: CoordFloat + HasKernel + Into<f64>>(
        &self,
        filename: P,
        tree: &RTree<ND, T>,
        index: Index,
    ) {
        draw_tree(filename, tree, index, self);
    }
}

pub fn draw_tree<P: AsRef<Path>, ND, T>(
    filename: P,
    tree: &RTree<ND, T>,
    index: Index,
    options: &TreeRenderOptions,
) where
    T: CoordFloat + HasKernel + Into<f64>,
{
    let mut img = RgbImage::new(options.width, options.height);
    let mut dirty = false;

    render_node(&mut img, &mut dirty, tree, index, 0, options.threshold);

    // only render an image if theres actually something to render
    if dirty {
        img.save(filename.as_ref()).unwrap();
    }
}

const BUFFER_WIDTH: f64 = 1.0;

fn render_node<ND, T>(
    canvas: &mut RgbImage,
    dirty: &mut bool,
    tree: &RTree<ND, T>,
    index: Index,
    level: usize,
    threshold: Option<usize>,
) where
    T: CoordFloat + HasKernel + Into<f64>,
{
    // If a threshold is set and we exceed it, stop rendering.
    if let Some(threshold) = threshold {
        if level > threshold {
            return;
        }
    }

    // Render all children of this node
    for child_index in tree.get_node(index).child_index_iter() {
        render_node(canvas, dirty, tree, child_index, level + 1, threshold);
    }

    // If we don't have a threshold our we are at the given threshold, render
    // the MBR for this ode.
    if threshold.is_none() || threshold == Some(level) {
        *dirty = true;
        draw_mbr(canvas, tree.get_node(index).get_region(), level);
    }
}

fn draw_line<C: Canvas<Pixel = Rgb<u8>>>(
    canvas: &mut C,
    level: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) {
    let colors = vec![
        Rgb([128u8, 21u8, 21u8]),
        Rgb([40u8, 180u8, 120u8]),
        Rgb([59u8, 49u8, 118u8]),
        Rgb([170u8, 108u8, 57u8]),
        Rgb([86u8, 119u8, 20u8]),
        Rgb([70u8, 50u8, 160u8]),
    ];

    draw_line_segment_mut(
        canvas,
        (x0 as f32, y0 as f32),
        (x1 as f32, y1 as f32),
        colors[level % colors.len()],
    );
}

fn draw_mbr<C: Canvas<Pixel = Rgb<u8>>, T: CoordFloat + Into<f64>>(
    canvas: &mut C,
    mbr: Rect<T>,
    level: usize,
) {
    let (x0, x1) = mbr.min().x_y();
    let (x0, x1) = (x0.into(), x1.into());
    let (y0, y1) = mbr.max().x_y();
    let (y0, y1) = (y0.into(), y1.into());

    draw_line(
        canvas,
        level,
        x0 + (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
        x0 + (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x0 + (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
        x1 - (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x1 - (BUFFER_WIDTH * level as f64),
        y1 - (BUFFER_WIDTH * level as f64),
        x1 - (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
    );
    draw_line(
        canvas,
        level,
        x1 - (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
        x0 + (BUFFER_WIDTH * level as f64),
        y0 + (BUFFER_WIDTH * level as f64),
    );
}
