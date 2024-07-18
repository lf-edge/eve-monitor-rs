use std::borrow::BorrowMut;

use crate::traits::IWindow;

pub struct LayerStack {
    layers: Vec<Box<dyn IWindow>>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }
    pub fn push(&mut self, mut layer: Box<dyn IWindow>) {
        layer.set_focus();
        // clear focus on current top layer
        if let Some(top) = self.layers.last_mut() {
            top.clear_focus();
        }
        self.layers.push(layer);
    }
    pub fn pop(&mut self) -> Option<Box<dyn IWindow>> {
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
    pub fn last_mut(&mut self) -> Option<&mut Box<dyn IWindow>> {
        self.layers.last_mut()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Box<dyn IWindow>> {
        self.layers.iter_mut()
    }
}

impl Clone for LayerStack {
    fn clone(&self) -> Self {
        Self::new()
    }
}
