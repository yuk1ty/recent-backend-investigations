use sea_orm::entity::prelude::*;
use super::_entities::todos::{ActiveModel, Entity};
pub type Todos = Entity;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}
