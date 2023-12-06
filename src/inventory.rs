use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub struct InventoryPlugin<IT: Component + CommandVisualBuilder> {
    _item_type: PhantomData<IT>,
}

impl<IT: Component + CommandVisualBuilder> Default for InventoryPlugin<IT> {
    fn default() -> Self {
        Self {
            _item_type: Default::default(),
        }
    }
}

impl<IT: Component + CommandVisualBuilder> Plugin for InventoryPlugin<IT> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                item_create_visual::<IT>,
                apply_deferred,
                item_reposition::<IT>,
            )
                .chain(),
        );
    }
}

pub trait CommandVisualBuilder {
    type C: EntityCommand;
    fn command_to_create_visual(&self) -> Self::C;
}

#[derive(Component)]
pub struct MarkerItemVisual;

#[derive(Component)]
pub struct Inventory<IT: Component + CommandVisualBuilder> {
    /// entities contained here have a MarkerItem component, it handles logic
    /// their rendering is created via item_create_visual
    pub items: VecDeque<Entity>,
    pub _item_type: PhantomData<IT>,
}

impl<IT: Component + CommandVisualBuilder> Default for Inventory<IT> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            _item_type: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct InventoryVisualDef {
    pub positions: Vec<Vec3>,
}

fn item_create_visual<IT: Component + CommandVisualBuilder>(
    mut commands: Commands,
    inventory: Query<(&Inventory<IT>, &InventoryVisualDef), Changed<Inventory<IT>>>,
    items_without_visual: Query<(Entity, &IT), Without<MarkerItemVisual>>,
) {
    for (inventory, visual_def) in inventory.iter() {
        for item in inventory.items.iter().take(visual_def.positions.len()) {
            let Ok(item) = items_without_visual.get(*item) else {
                continue;
            };
            let mut c = commands.entity(item.0);
            c.add(item.1.command_to_create_visual())
                .insert(MarkerItemVisual);
        }
    }
}
fn item_reposition<IT: Component + CommandVisualBuilder>(
    inventory: Query<(&Inventory<IT>, &InventoryVisualDef), Changed<Inventory<IT>>>,
    items_with_visual: Query<(Entity, &IT), With<MarkerItemVisual>>,
    mut q_transform: Query<&mut Transform>,
) {
    for (inventory, visual_def) in inventory.iter() {
        for (i, item) in inventory
            .items
            .iter()
            .take(visual_def.positions.len())
            .enumerate()
        {
            let Ok(item) = items_with_visual.get(*item) else {
                continue;
            };
            q_transform.get_mut(item.0).unwrap().translation = visual_def.positions[i];
        }
    }
}
