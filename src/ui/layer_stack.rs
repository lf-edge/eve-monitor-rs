use std::borrow::BorrowMut;

use crate::traits::IWindow;

pub struct LayerStack<A> {
    layers: Vec<Box<dyn IWindow<Action = A>>>,
}

impl<A> LayerStack<A> {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }
    pub fn push(&mut self, mut layer: Box<dyn IWindow<Action = A>>) {
        layer.set_focus();
        // clear focus on current top layer
        if let Some(top) = self.layers.last_mut() {
            top.clear_focus();
        }
        self.layers.push(layer);
    }
    pub fn pop(&mut self) -> Option<Box<dyn IWindow<Action = A>>> {
        let mut top = self.layers.pop();
        if let Some(layer) = top.borrow_mut() {
            layer.clear_focus();
        }
        // if there is still a layer set the focus
        if let Some(layer) = self.layers.last_mut() {
            layer.set_focus();
        }
        top
    }
    pub fn last_mut(&mut self) -> Option<&mut Box<dyn IWindow<Action = A>>> {
        self.layers.last_mut()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Box<dyn IWindow<Action = A>>> {
        self.layers.iter_mut()
    }
}
