use std::collections::HashMap;

use galfus_types::RealmId;

#[derive(Debug)]
pub struct RealmEntities<Camera, Light, Sprite, Shape> {
    pub cameras: HashMap<u32, Camera>,
    pub lights: HashMap<u32, Light>,
    pub sprites: HashMap<u32, Sprite>,
    pub shapes: HashMap<u32, Shape>,
}

#[derive(Debug)]
pub struct Realm2dState<Camera, Light, Sprite, Shape, Material> {
    pub entities: HashMap<RealmId, RealmEntities<Camera, Light, Sprite, Shape>>,
    pub materials: HashMap<u32, Material>,
}

impl<Camera, Light, Sprite, Shape> Default for RealmEntities<Camera, Light, Sprite, Shape> {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            lights: HashMap::new(),
            sprites: HashMap::new(),
            shapes: HashMap::new(),
        }
    }
}

impl<Camera, Light, Sprite, Shape, Material> Default
    for Realm2dState<Camera, Light, Sprite, Shape, Material>
{
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            materials: HashMap::new(),
        }
    }
}
