#![allow(dead_code)]
use super::{VerticalBox, HorizontalBox, LayoutNode, LayoutVariant, Alignment, Grid, Layout, ColorChange};
use std::cmp::{max, min};
use crate::{dimensions::*, MathFont};
use std::collections::BTreeMap;
use crate::parser::nodes;

pub struct VBox<'a, F> {
    pub width: Length<Px>,
    pub height: Length<Px>,
    pub depth: Length<Px>,
    node: VerticalBox<'a, F>,
}

impl<'a, F> Default for VBox<'a, F> {
    fn default() -> Self {
        Self {
            width:  Length::default(),
            height: Length::default(),
            depth:  Length::default(),
            node:   VerticalBox::default(),
        }
    }
}

impl<'a> VBox<'a, MathFont> {
    pub fn new() -> VBox<'a, MathFont> {
        VBox::default()
    }

    pub fn insert_node(&mut self, idx: usize, node: LayoutNode<'a, MathFont>) {
        self.width = max(self.width, node.width);
        self.height += node.height;
        self.node.contents.insert(idx, node);
    }

    pub fn add_node(&mut self, node: LayoutNode<'a, MathFont>) {
        self.width = max(self.width, node.width);
        self.height += node.height;
        self.node.contents.push(node);
    }

    pub fn set_offset(&mut self, offset: Length<Px>) {
        self.node.offset = offset;
    }

    pub fn build(mut self) -> LayoutNode<'a, MathFont> {
        // The depth only depends on the depth
        // of the last element and offset.
        if let Some(node) = self.node.contents.last() {
            self.depth = node.depth;
        }

        self.depth -= self.node.offset;
        self.height -= self.node.offset;

        LayoutNode {
            width: self.width,
            height: self.height,
            depth: self.depth,
            node: LayoutVariant::VerticalBox(self.node),
        }
    }
}

macro_rules! vbox {
    (offset: $offset:expr; $($node:expr),*) => ({
        let mut _vbox = builders::VBox::new();
        $( _vbox.add_node($node); )*
        _vbox.set_offset($offset);
        _vbox.build()
    });

    ( $($node:expr),* ) => ({
        let mut _vbox = builders::VBox::new();
        $( _vbox.add_node($node); )*
        _vbox.build()
    });
}

pub struct HBox<'a, F> {
    pub width: Length<Px>,
    pub height: Length<Px>,
    pub depth: Length<Px>,
    pub node: HorizontalBox<'a, F>,
    pub alignment: Alignment,
}

// NOTE: A limitation on derive(Clone, Default) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/
impl<'a, F> Default for HBox<'a, F> {
    fn default() -> Self {
        Self {
            width:     Length::default(),
            height:    Length::default(),
            depth:     Length::default(),
            alignment: Alignment::default(),
            node:      HorizontalBox::default(),
        }
    }
}



impl<'a> HBox<'a, MathFont> {
    pub fn new() -> HBox<'a, MathFont> {
        HBox::default()
    }

    pub fn add_node(&mut self, node: LayoutNode<'a, MathFont>) {
        self.width += node.width;
        self.height = max(self.height, node.height);
        self.depth = min(self.depth, node.depth);
        self.node.contents.push(node);
    }

    pub fn set_offset(&mut self, offset: Length<Px>) {
        self.node.offset = offset;
    }

    pub fn set_alignment(&mut self, align: Alignment) {
        self.node.alignment = align;
    }

    pub fn set_width(&mut self, width: Length<Px>) {
        self.width = width;
    }

    pub fn build(mut self) -> LayoutNode<'a, MathFont> {
        self.depth -= self.node.offset;
        self.height -= self.node.offset;

        LayoutNode {
            width: self.width,
            height: self.height,
            depth: self.depth,
            node: LayoutVariant::HorizontalBox(self.node),
        }
    }
}

impl<'a> Grid<'a, MathFont> {
    pub fn new() -> Grid<'a, MathFont> {
        Grid {
            contents: BTreeMap::new(),
            rows: Vec::new(),
            columns: Vec::new(),
        }
    }
    pub fn insert(&mut self, row: usize, column: usize, node: LayoutNode<'a, MathFont>) {
        if row >= self.rows.len() {
            self.rows.resize(row + 1, (Length::zero(), Length::zero()));
        }
        if node.height > self.rows[row].0 {
            self.rows[row].0 = node.height;
        }
        if node.depth < self.rows[row].1 {
            self.rows[row].1 = node.depth;
        }
        if column >= self.columns.len() {
            self.columns.resize(column + 1, Length::zero());
        }
        if node.width > self.columns[column] {
            self.columns[column] = node.width;
        }

        self.contents.insert((row, column), node);
    }
    pub fn build(self) -> LayoutNode<'a, MathFont> {
        LayoutNode {
            width:  self.columns.iter().cloned().sum(),
            height: self.rows.iter().map(|&(height, depth)| height - depth).sum(),
            depth: Length::zero(),
            node: LayoutVariant::Grid(self)
        }
    }
    pub fn x_offsets(&self) -> Vec<Length<Px>> {
        self.columns.iter().scan(Length::zero(), |acc, &width| {
            let x = *acc;
            *acc += width;
            Some(x)
        }).collect()
    }
    pub fn y_offsets(&self) -> Vec<Length<Px>> {
        self.rows.iter().scan(Length::zero(), |acc, &(height, depth)| {
            let x = *acc;
            *acc += height - depth;
            Some(x)
        }).collect()
    }
}

macro_rules! hbox {
    (offset: $offset:expr; $($node:expr),*) => ({
        let mut _hbox = builders::HBox::new();
        $( _hbox.add_node($node); )*
        _hbox.set_offset($offset);
        _hbox.build()
    });

    (align: $align:expr; width: $width:expr; $($node:expr),*) => ({
        let mut _hbox = builders::HBox::new();
        let align = $align;
        let width = $width;
        $( _hbox.add_node($node); )*
        _hbox.set_alignment(align);
        _hbox.set_width(width);
        _hbox.build()
    });

    ( $($node:expr),* ) => ({
        let mut _hbox = builders::HBox::new();
        $( _hbox.add_node($node); )*
        _hbox.build()
    });
}

macro_rules! rule {
    (width: $width:expr, height: $height:expr) => (
        rule!(width: $width, height: $height, depth: Length::zero())
    );

    (width: $width:expr, height: $height:expr, depth: $depth:expr) => (
        LayoutNode {
            width:  $width,
            height: $height,
            depth:  $depth,
            node: LayoutVariant::Rule,
        }
    );
}

macro_rules! kern {
    (vert: $height:expr) => (
        LayoutNode {
            width:  Length::zero(),
            height: $height,
            depth:  Length::zero(),
            node:   LayoutVariant::Kern,
        }
    );

    (horz: $width:expr) => (
        LayoutNode {
            width:   $width,
            height: Length::zero(),
            depth:  Length::zero(),
            node:   LayoutVariant::Kern,
        }
    );
}

pub fn color<'a>(layout: Layout<'a, MathFont>, color: &nodes::Color) -> LayoutNode<'a, MathFont> {
    LayoutNode {
        width: layout.width,
        height: layout.height,
        depth: layout.depth,
        node: LayoutVariant::Color(ColorChange {
            color: color.color,
            inner: layout.contents,
        }),
    }
}
