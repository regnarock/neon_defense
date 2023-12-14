use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::marker::PhantomData;

/// This plugin handles the creation of Items in the inventory
pub struct InventoryPlugin<IT: Component + ItemSpriteBuilder> {
    _item_type: PhantomData<IT>,
}

impl<IT: Component + ItemSpriteBuilder> Default for InventoryPlugin<IT> {
    fn default() -> Self {
        Self {
            _item_type: Default::default(),
        }
    }
}

impl<IT: Component + ItemSpriteBuilder> Plugin for InventoryPlugin<IT> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                item_create_sprite::<IT>,
                apply_deferred,
                redraw_inventory_on_change::<IT>,
            )
                .chain(),
        );
    }
}

pub trait ItemSpriteBuilder {
    type C: EntityCommand;
    fn build_sprite(&self) -> Self::C;
}

#[derive(Component)]
struct MarkerItemSpriteBuilt;

#[derive(Component)]
pub struct Inventory<IT: Component + ItemSpriteBuilder> {
    /// entities contained here have a MarkerItem component, it handles logic
    /// their rendering is created via item_create_visual
    pub items: VecDeque<Entity>,
    pub positions: Vec<Vec3>,

    _item_type: PhantomData<IT>,
}

pub struct SpawnInventory<IT: Component + ItemSpriteBuilder> {
    items: Vec<Entity>,
    configuration: InventoryConfiguration,

    _item_type: PhantomData<IT>,
}

impl<IT> SpawnInventory<IT>
where
    IT: Component + ItemSpriteBuilder,
{
    pub fn new(items: Vec<Entity>, configuration: InventoryConfiguration) -> Self {
        Self {
            items,
            configuration,
            _item_type: Default::default(),
        }
    }
}

/// Configuration for the inventory
///   positions: Vec<Vec3> - positions of the items in the inventory
/// TODO: should be relative to the inventory entity/transform
pub struct InventoryConfiguration {
    pub positions: Vec<Vec3>,
}

impl<IT> EntityCommand for SpawnInventory<IT>
where
    IT: Component + ItemSpriteBuilder,
{
    fn apply(self, id: Entity, world: &mut World) {
        world.entity_mut(id).insert((Inventory::<IT> {
            items: self.items.into_iter().collect(),
            positions: self.configuration.positions,
            _item_type: self._item_type,
        },));
    }
}

fn item_create_sprite<IT: Component + ItemSpriteBuilder>(
    mut commands: Commands,
    inventory: Query<&Inventory<IT>, Changed<Inventory<IT>>>,
    items_without_visual: Query<(Entity, &IT), Without<MarkerItemSpriteBuilt>>,
) {
    for inventory in inventory.iter() {
        for item in inventory.items.iter().take(inventory.positions.len()) {
            if let Ok((entity, item)) = items_without_visual.get(*item) {
                let mut c = commands.entity(entity);
                c.add(item.build_sprite()).insert(MarkerItemSpriteBuilt);
            }
        }
    }
}

fn redraw_inventory_on_change<IT: Component + ItemSpriteBuilder>(
    inventory: Query<&Inventory<IT>, Changed<Inventory<IT>>>,
    mut items_with_visual: Query<&mut Transform, (With<MarkerItemSpriteBuilt>, With<IT>)>,
) {
    for inventory in inventory.iter() {
        for (i, &item) in inventory
            .items
            .iter()
            .take(inventory.positions.len())
            .enumerate()
        {
            if let Ok(mut transform) = items_with_visual.get_mut(item) {
                //TODO: should be relative to the inventory entity/transform
                transform.translation = inventory.positions[i];
            }
        }
    }
}
