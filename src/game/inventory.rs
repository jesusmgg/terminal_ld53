use anyhow::Result;

const MAX_INSTANCE_COUNT: usize = 128;

pub struct InventoryMgr {
    pub item_ammo_bullets: Vec<usize>,
    pub item_ammo_rockets: Vec<usize>,
    pub item_ammo_energy_cells: Vec<usize>,
}

impl InventoryMgr {
    pub fn new() -> Self {
        Self {
            item_ammo_bullets: Vec::with_capacity(MAX_INSTANCE_COUNT),
            item_ammo_rockets: Vec::with_capacity(MAX_INSTANCE_COUNT),
            item_ammo_energy_cells: Vec::with_capacity(MAX_INSTANCE_COUNT),
        }
    }

    /// Creates a new inventory instance with all elements set to zero.
    /// Returns the index of the newly created instance.
    pub fn add(&mut self) -> Result<usize> {
        self.item_ammo_bullets.push(0);
        self.item_ammo_rockets.push(0);
        self.item_ammo_energy_cells.push(0);

        let index = self.len() - 1;
        Ok(index)
    }

    pub fn len(&self) -> usize {
        self.item_ammo_bullets.len()
    }
}
