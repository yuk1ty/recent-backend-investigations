use sea_orm::entity::prelude::*;
use super::_entities::todo::{ActiveModel, Entity};
pub type Todo = Entity;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}
