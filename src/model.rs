use std::rc::Rc;

use crate::raw_model::RawModel;

pub struct Model {
    a: u32,
}

impl Model {
    pub fn update(&mut self, _: &RawModel) {
        // update the model
        self.a += 1;
    }
}

impl Default for Model {
    fn default() -> Self {
        Model { a: 0 }
    }
}

impl From<&Rc<RawModel>> for Model {
    fn from(rawmodel: &Rc<RawModel>) -> Self {
        Model { a: 0 }
    }
}
