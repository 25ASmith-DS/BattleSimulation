use graphics::types::Vec2d;



pub struct Soldier {
    level: SoldierType,
    pos: Vec2d
}
impl Soldier {
    pub fn new() -> Self {
        Self { level: SoldierType::Retiarii, pos: [0.0, 0.0] }
    }

    pub fn test() {
        
    }
}

pub enum SoldierType {
    Retiarii
}