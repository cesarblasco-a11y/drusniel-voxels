use bevy::prelude::*;
use std::collections::HashMap;

/// Types of items that can be collected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Fur,
}

/// Player inventory resource
#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub items: HashMap<ItemType, u32>,
}

impl Inventory {
    pub fn add_item(&mut self, item_type: ItemType) {
        *self.items.entry(item_type).or_insert(0) += 1;
    }

    pub fn get_count(&self, item_type: ItemType) -> u32 {
        *self.items.get(&item_type).unwrap_or(&0)
    }

    pub fn remove_item(&mut self, item_type: ItemType, count: u32) -> bool {
        if let Some(current) = self.items.get_mut(&item_type) {
            if *current >= count {
                *current -= count;
                return true;
            }
        }
        false
    }
}

/// Component for item drops
#[derive(Component)]
pub struct ItemDrop {
    pub item_type: ItemType,
    pub position: Vec3,
}
