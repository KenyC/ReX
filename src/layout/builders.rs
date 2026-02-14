#![allow(dead_code)]
use super::{VerticalBox, HorizontalBox, LayoutNode, LayoutVariant, Alignment, Grid, Layout, ColorChange};

use crate::dimensions::{units::Px, Unit};
use std::collections::BTreeMap;
use crate::parser::nodes;

pub struct VBox<'a, F> {
    pub width:  Unit<Px>,
    pub height: Unit<Px>,
    pub depth:  Unit<Px>,
    node: VerticalBox<'a, F>,
}

impl<'a, F> Default for VBox<'a, F> {
    fn default() -> Self {
        Self {
            width:  Unit::ZERO,
            height: Unit::ZERO,
            depth:  Unit::ZERO,
            node:   VerticalBox::default(),
        }
    }
}

impl<'a, F> VBox<'a, F> {
    pub fn new() -> VBox<'a, F> {
        VBox::default()
    }

    pub fn insert_node(&mut self, idx: usize, node: LayoutNode<'a, F>) {
        self.width = Unit::max(self.width, node.width);
        self.height += node.height;
        self.node.contents.insert(idx, node);
    }

    pub fn add_node(&mut self, node: LayoutNode<'a, F>) {
        self.width = Unit::max(self.width, node.width);
        self.height += node.height;
        self.node.contents.push(node);
    }

    pub fn set_offset(&mut self, offset: Unit<Px>) {
        self.node.offset = offset;
    }

    pub fn build(mut self) -> LayoutNode<'a, F> {
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
    pub width: Unit<Px>,
    pub height: Unit<Px>,
    pub depth: Unit<Px>,
    pub node: HorizontalBox<'a, F>,
    pub alignment: Alignment,
}

// NOTE: A limitation on derive(Clone, Default) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/
impl<'a, F> Default for HBox<'a, F> {
    fn default() -> Self {
        Self {
            width:     Unit::ZERO,
            height:    Unit::ZERO,
            depth:     Unit::ZERO,
            alignment: Alignment::default(),
            node:      HorizontalBox::default(),
        }
    }
}



impl<'a, F> HBox<'a, F> {
    pub fn new() -> HBox<'a, F> {
        HBox::default()
    }

    pub fn add_node(&mut self, node: LayoutNode<'a, F>) {
        self.width += node.width;
        self.height = Unit::max(self.height, node.height);
        self.depth = Unit::min(self.depth, node.depth);
        self.node.contents.push(node);
    }

    pub fn set_offset(&mut self, offset: Unit<Px>) {
        self.node.offset = offset;
    }

    pub fn set_alignment(&mut self, align: Alignment) {
        self.node.alignment = align;
    }

    pub fn set_width(&mut self, width: Unit<Px>) {
        self.width = width;
    }

    pub fn build(mut self) -> LayoutNode<'a, F> {
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

impl<'a, F> Grid<'a, F> {
    /// Creates a new [`Grid`](crate::layout::Grid)
    pub fn new() -> Grid<'a, F> {
        Grid {
            contents: BTreeMap::new(),
            rows: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Insert node in the grid at the given position.
    /// Replaces any node there may have been in this position.
    pub fn insert(&mut self, row: usize, column: usize, node: LayoutNode<'a, F>) {
        // If replacing an existing node, we may need to recalculate row/column dimensions
        if let Some(old_node) = self.contents.get(&(row, column)) {
            let old_height = old_node.height;
            let old_depth = old_node.depth;
            let old_width = old_node.width;

            // Check if old node defined the row height or depth
            if old_height == self.rows[row].0 || old_depth == self.rows[row].1 {
                // Recalculate row dimensions from all nodes in this row (excluding old node)
                let (max_height, min_depth) = self.contents.iter()
                    .filter(|&(&(r, c), _)| r == row && c != column)
                    .fold((Unit::ZERO, Unit::ZERO), |(h, d), (_, n)| {
                        (Unit::max(h, n.height), Unit::min(d, n.depth))
                    });
                self.rows[row] = (max_height, min_depth);
            }

            // Check if old node defined the column width
            if old_width == self.columns[column] {
                // Recalculate column width from all nodes in this column (excluding old node)
                let max_width = self.contents.iter()
                    .filter(|&(&(r, c), _)| c == column && r != row)
                    .fold(Unit::ZERO, |w, (_, n)| Unit::max(w, n.width));
                self.columns[column] = max_width;
            }
        }

        // Ensure row/column vectors are large enough
        if row >= self.rows.len() {
            self.rows.resize(row + 1, (Unit::ZERO, Unit::ZERO));
        }
        if column >= self.columns.len() {
            self.columns.resize(column + 1, Unit::ZERO);
        }

        // Update dimensions with new node
        if node.height > self.rows[row].0 {
            self.rows[row].0 = node.height;
        }
        if node.depth < self.rows[row].1 {
            self.rows[row].1 = node.depth;
        }
        if node.width > self.columns[column] {
            self.columns[column] = node.width;
        }

        self.contents.insert((row, column), node);
    }

    /// Convert [`Grid`](crate::layout::Grid) into [`LayoutNode`](crate::layout::LayoutNode)
    pub fn build(self) -> LayoutNode<'a, F> {
        LayoutNode {
            width:  self.columns.iter().cloned().sum(),
            height: self.rows.iter().map(|&(height, depth)| height - depth).sum(),
            depth: Unit::ZERO,
            node: LayoutVariant::Grid(self)
        }
    }

    /// Returns, for every column, the sum of the widths of the preceding columns.
    pub fn x_offsets(&self) -> Vec<Unit<Px>> {
        self.columns.iter().scan(Unit::ZERO, |acc, &width| {
            let x = *acc;
            *acc += width;
            Some(x)
        }).collect()
    }

    /// Returns, for every row, the sum of the heights of the preceding rows.
    pub fn y_offsets(&self) -> Vec<Unit<Px>> {
        self.rows.iter().scan(Unit::ZERO, |acc, &(height, depth)| {
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
        rule!(width: $width, height: $height, depth: Unit::ZERO)
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

pub fn color<'a, F>(layout: Layout<'a, F>, color: &nodes::Color) -> LayoutNode<'a, F> {
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
