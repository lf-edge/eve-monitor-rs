use crate::raw_model::RawModel;

#[derive(Debug)]
pub struct Model {}

impl Model {}

impl Default for Model {
    fn default() -> Self {
        Model {}
    }
}

impl From<&RawModel> for Model {
    fn from(_raw_model: &RawModel) -> Self {
        Model {}
    }
}
