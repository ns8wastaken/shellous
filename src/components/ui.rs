use crate::components::arena::{Arena, Slot};
use crate::components::base::align::Align;
use crate::components::base::column::ColumnNode;
use crate::components::base::group::Group;
use crate::components::base::padding::Padding;
use crate::components::base::rect::RectNode;
use crate::components::base::row::RowNode;
use crate::components::base::text::TextNode;
use crate::components::base::stacks::{stack_horizontal, stack_vertical};
use crate::components::layout_tree::LayoutNode;
use crate::components::geom::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::text::TextStyle;
use crate::shell::event::ShellEvent;
use crate::shell::state::ShellState;

pub type ElementArena = Arena<Node>;

pub enum Node {
    Rect(RectNode),
    Text(TextNode),
    Row(RowNode),
    Column(ColumnNode),
    Group(Group),
    Align(Align),
    Padding(Padding),
}

impl Node {
    pub fn children(&self) -> &[Slot] {
        match self {
            Self::Rect(_) | Self::Text(_) => &[],
            Self::Row(r) => &r.children,
            Self::Column(c) => &c.children,
            Self::Group(g) => &g.children,
            Self::Align(a) => std::slice::from_ref(&a.child),
            Self::Padding(p) => std::slice::from_ref(&p.child),
        }
    }

    pub fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        match self {
            Self::Rect(r) => r.size,
            Self::Text(t) => {
                // ponytail: font measurement stubbed; returns a fixed size.
                // Proper measurement will use fontdue::Font in Phase 6.
                let w = t.text.len() as f32 * t.font_size * 0.6;
                Size::new(w, t.font_size * 1.2)
            }
            Self::Row(r) => {
                let mut total_w = 0.0f32;
                let mut max_h = 0.0f32;
                for &slot in &r.children {
                    let child = arena.get(slot).unwrap();
                    let size = child.layout(available, cache, arena);
                    total_w += size.w;
                    if size.h > max_h { max_h = size.h; }
                }
                if !r.children.is_empty() {
                    total_w += r.spacing * (r.children.len() - 1) as f32;
                }
                Size::new(total_w, max_h)
            }
            Self::Column(c) => {
                let mut total_h = 0.0f32;
                let mut max_w = 0.0f32;
                for &slot in &c.children {
                    let child = arena.get(slot).unwrap();
                    let size = child.layout(available, cache, arena);
                    total_h += size.h;
                    if size.w > max_w { max_w = size.w; }
                }
                if !c.children.is_empty() {
                    total_h += c.spacing * (c.children.len() - 1) as f32;
                }
                Size::new(max_w, total_h)
            }
            Self::Group(g) => {
                let mut mw = 0.0f32;
                let mut mh = 0.0f32;
                for &slot in &g.children {
                    let size = arena.get(slot).unwrap().layout(available, cache, arena);
                    if size.w > mw { mw = size.w; }
                    if size.h > mh { mh = size.h; }
                }
                Size::new(mw, mh)
            }
            Self::Align(a) => {
                let child = arena.get(a.child).unwrap();
                child.layout(available, cache, arena)
            }
            Self::Padding(p) => {
                let cw = (available.w - p.left - p.right).max(0.0);
                let ch = (available.h - p.top - p.bottom).max(0.0);
                let child = arena.get(p.child).unwrap();
                child.layout(Size::new(cw, ch), cache, arena)
            }
        }
    }

    pub fn layout_tree(
        &self,
        rect: Rect,
        cache: &AnimationCache,
        arena: &ElementArena,
    ) -> LayoutNode {
        match self {
            Self::Rect(_) | Self::Text(_) => LayoutNode::new(rect),
            Self::Row(r) => {
                let child_sizes: Vec<Size> = r.children.iter()
                    .map(|&slot| arena.get(slot).unwrap().layout(rect.size(), cache, arena))
                    .collect();
                let child_rects = stack_horizontal(rect, &child_sizes, r.spacing);
                LayoutNode {
                    rect,
                    children: child_rects.iter().zip(&r.children)
                        .map(|(&child_rect, &slot)| {
                            arena.get(slot).unwrap().layout_tree(child_rect, cache, arena)
                        })
                        .collect(),
                }
            }
            Self::Column(c) => {
                let child_sizes: Vec<Size> = c.children.iter()
                    .map(|&slot| arena.get(slot).unwrap().layout(rect.size(), cache, arena))
                    .collect();
                let child_rects = stack_vertical(rect, &child_sizes, c.spacing);
                LayoutNode {
                    rect,
                    children: child_rects.iter().zip(&c.children)
                        .map(|(&child_rect, &slot)| {
                            arena.get(slot).unwrap().layout_tree(child_rect, cache, arena)
                        })
                        .collect(),
                }
            }
            Self::Group(g) => LayoutNode {
                rect,
                children: g
                    .children
                    .iter()
                    .map(|&slot| {
                        arena
                            .get(slot)
                            .unwrap()
                            .layout_tree(rect, cache, arena)
                    })
                    .collect(),
            },
            Self::Align(a) => {
                let child = arena.get(a.child).unwrap();
                let child_size = child.layout(rect.size(), cache, arena);
                let child_rect = a.alignment.apply(rect, child_size);
                LayoutNode {
                    rect,
                    children: vec![child.layout_tree(child_rect, cache, arena)],
                }
            }
            Self::Padding(p) => {
                let inner = rect.inset(p.left, p.top, p.right, p.bottom);
                let child = arena.get(p.child).unwrap();
                LayoutNode {
                    rect,
                    children: vec![child.layout_tree(inner, cache, arena)],
                }
            }
        }
    }

    pub fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        match self {
            Self::Rect(r) => {
                batch.push(node.rect, DrawParams::Rect(r.style.clone()));
            }
            Self::Text(t) => {
                batch.push(
                    node.rect,
                    DrawParams::Text(
                        TextStyle::new()
                            .text(t.text.clone())
                            .size(t.font_size)
                            .color(t.color),
                    ),
                );
            }
            Self::Row(r) => {
                for (&slot, child_node) in r.children.iter().zip(&node.children) {
                    ctx.arena.get(slot).unwrap().draw(child_node, batch, ctx);
                }
            }
            Self::Column(c) => {
                for (&slot, child_node) in c.children.iter().zip(&node.children) {
                    ctx.arena.get(slot).unwrap().draw(child_node, batch, ctx);
                }
            }
            Self::Group(g) => {
                for (&slot, child_node) in g.children.iter().zip(&node.children) {
                    ctx.arena.get(slot).unwrap().draw(child_node, batch, ctx);
                }
            }
            Self::Align(a) => {
                ctx.arena
                    .get(a.child)
                    .unwrap()
                    .draw(&node.children[0], batch, ctx);
            }
            Self::Padding(p) => {
                ctx.arena
                    .get(p.child)
                    .unwrap()
                    .draw(&node.children[0], batch, ctx);
            }
        }
    }

    pub fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        match self {
            Self::Rect(r) => {
                if node.rect.contains(x, y) {
                    if let Some(ref f) = r.on_click {
                        f();
                    }
                    true
                } else {
                    false
                }
            }
            Self::Text(_) => false,
            Self::Row(r) => {
                for (&slot, child_node) in r.children.iter().zip(&node.children).rev() {
                    if child_node.rect.contains(x, y)
                        && ctx
                            .arena
                            .get(slot)
                            .unwrap()
                            .on_click(child_node, x, y, ctx)
                    {
                        return true;
                    }
                }
                false
            }
            Self::Column(c) => {
                for (&slot, child_node) in c.children.iter().zip(&node.children).rev() {
                    if child_node.rect.contains(x, y)
                        && ctx
                            .arena
                            .get(slot)
                            .unwrap()
                            .on_click(child_node, x, y, ctx)
                    {
                        return true;
                    }
                }
                false
            }
            Self::Group(g) => {
                for (&slot, child_node) in g.children.iter().zip(&node.children).rev() {
                    if child_node.rect.contains(x, y)
                        && ctx
                            .arena
                            .get(slot)
                            .unwrap()
                            .on_click(child_node, x, y, ctx)
                    {
                        return true;
                    }
                }
                false
            }
            Self::Align(a) => ctx
                .arena
                .get(a.child)
                .unwrap()
                .on_click(&node.children[0], x, y, ctx),
            Self::Padding(p) => ctx
                .arena
                .get(p.child)
                .unwrap()
                .on_click(&node.children[0], x, y, ctx),
        }
    }
}

pub trait Controller {
    fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache, arena: &mut ElementArena) -> bool;
    fn sync(&self, now: f32, cache: &AnimationCache, arena: &mut ElementArena, surface_w: f32, surface_h: f32);
    fn on_click(&self, x: f32, y: f32, arena: &ElementArena, layout: &LayoutNode, ctx: &RenderContext) -> bool {
        let _ = (x, y, arena, layout, ctx);
        false
    }
}

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
    pub animations: &'a AnimationCache,
    pub arena: &'a ElementArena,
}
