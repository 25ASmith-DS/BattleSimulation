


use std::f64::consts::{TAU, PI};

use graphics::{clear, Transformed, rectangle::{centered_square}, types::Color};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::RenderArgs;

pub const SCREEN_WIDTH: f64 = 1280.0;
pub const SCREEN_HEIGHT: f64 = 720.0;

const GRASS_GREEN: [f32; 4] = [0.42, 0.66, 0.2, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.1];
const SOLDIER_SIZE: f64 = 4.0;
const MAX_HEALTH: u32 = 50;

use rand::random;

pub struct App {
    gl: GlGraphics,
    frame: u64,
    pub soldiers: Vec<Soldier>
}

impl App {

    pub fn add_soldier(&mut self, team: Team, legion: SoldierType, pos: [f64; 2]) {
        let id: u64 = self.soldiers.len() as u64;
        self.soldiers.push(Soldier::new(team, legion, pos, match legion {
            SoldierType::Triarii => {35 * 60}            
            SoldierType::Principes => {20 * 60}            
            SoldierType::Hastati => {7 * 60}            
            SoldierType::Velites => {10}            
        }, id));
    }

    pub fn new(opengl: OpenGL) -> Self {
        Self {
            gl: GlGraphics::new(opengl),
            frame: 0,
            soldiers: Vec::new(),
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let soldiers = self.soldiers.clone();

        self.gl.draw(args.viewport(), |c, g| {
            // Clear the screen.
            clear(GRASS_GREEN, g);

            for s in soldiers.iter() {
                let t = c.transform.clone()
                    .trans(s.x(), s.y());
                    let rect = centered_square(0.0, 0.0, SOLDIER_SIZE);
                    graphics::rectangle(match s.alive() {
                        true => s.color(),
                        false => BLACK
                    }, rect, t, g);
            }

        });
    }

    pub fn update(&mut self) {

        
        let reds = self.soldiers.iter()
            .filter(|s| 
                s.alive() && s.team == Team::Red
            ).count();

        let blues = self.soldiers.iter()
            .filter(|s| 
                s.alive() && s.team == Team::Blue
            ).count();

        //println!("Red: {}\nBlue: {}", reds, blues);

        self.frame += 1;
        for i in 0..self.soldiers.len() {
            if self.soldiers[i].alive() {
                let mut others = self.soldiers
                    .iter_mut()
                    .collect::<Vec<&mut Soldier>>();
                let s = others.remove(i);
                s.update(others);
            }
        };
    }

    #[allow(dead_code)]
    pub fn frame(&self) -> u64 {
        self.frame
    }

}



#[derive(Clone, Debug)]
pub struct Soldier {
    legion: SoldierType,
    pos: [f64; 2],
    vel: [f64; 2],
    state: BattleState,
    team: Team,
    level: f64,
    health: u32,
    id: u64
}
#[allow(dead_code)]
impl Soldier {
    
    // Physics coefficients
    const ACCEL: f64 = 0.1;
    const DRAG: f64 = 0.97;
    const MAX_SPEED: f64 = 1.3;
    const PUSH_POWER: f64 = 0.6;

    pub fn new(team: Team, legion: SoldierType, pos: [f64; 2], wait_frames: u32, id: u64) -> Self {
        Self {
            legion,
            pos: [pos[0] + random::<f64>() * 2.0, pos[1] + random::<f64>() * 2.0],
            vel: [0.0, 0.0],
            state: BattleState::Idle(wait_frames),
            team,
            level: random::<f64>() * 2.0 + 1.0,
            health: legion.base_health() + (random::<u32>() % 10) - 5,
            id
        }
    }

    pub fn update(&mut self, mut others: Vec<&mut Soldier>) {

        if self.x() > SCREEN_WIDTH {
            self.add_velocity(-10.0, 0.0);
        }
        if self.x() < 0.0 {
            self.add_velocity(10.0, 0.0);
        }
        if self.y() > SCREEN_HEIGHT {
            self.add_velocity(0.0, -10.0);
        }
        if self.y() < 0.0 {
            self.add_velocity(0.0, 10.0);
        }



        // Update position
        self.pos = [
            self.x() + self.legion.speed_multiplier() * self.x_vel().min(Soldier::MAX_SPEED).max(-Soldier::MAX_SPEED),
            self.y() + self.legion.speed_multiplier() * self.y_vel().min(Soldier::MAX_SPEED).max(-Soldier::MAX_SPEED),
        ];

        // Update velocity

        let colliders = others.iter_mut().filter(|other|
            self.inside(other) && other.alive()
        ).collect::<Vec<&mut &mut Soldier>>();

        for c in colliders {
            let angle = self.angle_to(c);
            let pushx = angle.cos() * (Soldier::PUSH_POWER) * (6.0 - self.distance_from(c)).max(0.0);
            let pushy = angle.sin() * (Soldier::PUSH_POWER) * (6.0 - self.distance_from(c)).max(0.0);
            self.add_velocity(-pushx, -pushy);
            c.add_velocity(pushx, pushy)
        };

        self.vel = [
            self.x_vel() * Soldier::DRAG,
            self.y_vel() * Soldier::DRAG,
        ];


        self.state = match self.state {
            BattleState::Dead => { BattleState::Dead }
            BattleState::Idle(t) => {
                if t == 0 {
                    // Filter for targetable soldiers
                    let alive_enemies = others
                        .into_iter()
                        .filter(|soldier| 
                            soldier.alive() && // Target must be alive
                            soldier.team != self.team // Target must not be teammate
                        );
                    
                    // Search for closest available target
                    let mut closest_opponent: Option<(&mut Soldier, f64)> = None;
                    for s in alive_enemies {
                        if let Some((_, prev_dist)) = closest_opponent {
                            let dist = self.distance_from(s);
                            if dist < prev_dist {
                                closest_opponent = Some((s, dist))
                            }
                        } else {
                            let dist = self.distance_from(s);
                            closest_opponent = Some((s, dist));
                        }
                    };

                    // Target closest, if available
                    if let Some((soldier, _)) = closest_opponent {
                        BattleState::Charging(10, soldier)
                    } else {
                        BattleState::Idle(5)
                    }
                } else {
                    BattleState::Idle(t - 1)
                }
            }
            BattleState::Charging(t, target) => {
                let target = unsafe {&mut *target};
                if target.dead() {
                    BattleState::Idle(5)
                } else if t == 0 {
                    BattleState::Idle(2)
                } else {
                    let angle_to_target = self.angle_to(target);

                    // Accelerate towards target
                    self.vel = [
                        self.x_vel() + angle_to_target.cos() * Soldier::ACCEL,
                        self.y_vel() + angle_to_target.sin() * Soldier::ACCEL,
                    ];

                    
                    // Fight if within range of target
                    if self.distance_from(target) < SOLDIER_SIZE * 2.0 * self.legion.range_multiplier() {
                        let battle_time = ((random::<f64>() * 60.0) + 60.0) as u32;
                        
                        target.state = BattleState::Defending(battle_time, self);
                        BattleState::Attacking(battle_time, target)  
                    } else {
                        BattleState::Charging(t - 1, target)
                    }
                }
            }
            BattleState::Attacking(t, target) => {
                let target = unsafe {&mut *target};
                
                if target.dead() {
                    BattleState::Idle(5)
                } else if t > 0 {
                    self.vel = [
                        self.x_vel() * 0.7,
                        self.y_vel() * 0.7,
                    ];
                    BattleState::Attacking(t - 1, target)
                } else /* t == 0 */{

                    let l1 = self.level * self.battle_multiplier();
                    let l2 = target.level * target.battle_multiplier();

                    let winning_chance = l1 / (l1 + l2);
                    let outcome = random::<f64>();

                    let battle_won = outcome < winning_chance;

                    // println!("{} {} (lv {:.3}) attacking {} {} (lv {:.3})",
                    //     self.team.to_string(),
                    //     self.id,
                    //     l1,
                    //     target.team.to_string(),
                    //     target.id,
                    //     l2
                    // );

                    if !battle_won {
                        self.health = 0;
                        BattleState::Dead
                    } else {
                        match self.health.checked_sub(3) {
                            Some(0) => {
                                target.state = BattleState::Dead;
                                self.health = 0;
                                BattleState::Dead
                            }
                            Some(x) => {
                                target.state = BattleState::Dead;
                                self.health = x;
                                BattleState::Idle(5)
                            }
                            None => {
                                BattleState::Dead
                            }
                        }
                    }
                }
            }
            BattleState::Defending(t, attacker) => {
                let attacker = unsafe {&mut *attacker};                
                if attacker.dead() {
                    BattleState::Idle(5)
                } else if t > 0 {
                    self.vel = [
                        self.x_vel() * 0.7,
                        self.y_vel() * 0.7,
                    ];
                    BattleState::Defending(t - 1, attacker)
                } else /* t == 0 */ {
                    BattleState::Idle(5)
                }
            }
        }
    }

    pub fn color(&self) -> Color {
        match self.team {
            Team::Blue => {
                match self.legion {
                    SoldierType::Triarii => {[0., 0., 125./255., 1.]}
                    SoldierType::Principes => {[32./255., 32./255., 150./255., 1.]}
                    SoldierType::Hastati => {[65./255., 65./255., 175./255., 1.]}
                    SoldierType::Velites => {[100./255., 100./255., 200./255., 1.]}
                }
            }
            Team::Red => {
                match self.legion {
                    SoldierType::Triarii => {[125./255., 0., 0., 1.]}
                    SoldierType::Principes => {[150./255., 32./255., 32./255., 1.]}
                    SoldierType::Hastati => {[175./255., 65./255., 65./255., 1.]}
                    SoldierType::Velites => {[200./255., 100./255., 100./255., 1.]}
                }
            }
        }
    }

    pub fn distance_from(&self, other: &Self) -> f64 {
        let dx = other.pos[0] - self.pos[0];
        let dy = other.pos[1] - self.pos[1];
        (dx*dx + dy*dy).sqrt()
    }
    pub fn angle_to(&self, other: &Self) -> f64 {
        (other.y() - self.y()).atan2(other.x() - self.x())
    }

    pub fn x(&self) -> f64 {self.pos[0]}
    pub fn y(&self) -> f64 {self.pos[1]}
    pub fn x_vel(&self) -> f64 {self.vel[0]}
    pub fn y_vel(&self) -> f64 {self.vel[1]}
    pub fn pos(&self) -> [f64; 2] {self.pos}
    pub fn pos_centered(&self) -> [f64; 2] {
        [
            self.x() + (SOLDIER_SIZE / 2.0),
            self.y() + (SOLDIER_SIZE / 2.0),
        ]
    }
    
    pub fn add_velocity(&mut self, x: f64, y: f64) {
        self.vel = [
            self.x_vel() + x,
            self.y_vel() + y
        ];
    }

    pub fn alive(&self) -> bool {
        self.state != BattleState::Dead
    }
    pub fn dead(&self) -> bool {
        self.state == BattleState::Dead
    }
    pub fn inside(&self, other: &Self) -> bool {
        (self.x() - other.x()).abs() < SOLDIER_SIZE &&
        (self.y() - other.y()).abs() < SOLDIER_SIZE
    }

    pub fn battle_multiplier(&self) -> f64 {
        self.legion.battle_bonus() * self.health as f64 / self.legion.base_health() as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoldierType {
    Triarii,
    Principes,
    Hastati,
    Velites,
} impl SoldierType {
    fn battle_bonus(&self) -> f64 {
        match self {
            Self::Triarii => {1.6}
            Self::Principes => {1.3}
            Self::Hastati => {0.9}
            Self::Velites => {1.0}
        }
    }
    fn range_multiplier(&self) -> f64 {
        match self {
            Self::Triarii => {1.4}
            Self::Principes => {1.8}
            Self::Hastati => {1.0}
            Self::Velites => {2.2}
        }
    }
    fn speed_multiplier(&self) -> f64 {
        match self {
            Self::Triarii => {0.7}
            Self::Principes => {1.2}
            Self::Hastati => {1.2}
            Self::Velites => {1.5}
        }
    }
    fn base_health(&self) -> u32 {
        match self {
            Self::Triarii => {MAX_HEALTH}
            Self::Principes => {MAX_HEALTH - 10}
            Self::Hastati => {MAX_HEALTH - 20}
            Self::Velites => {MAX_HEALTH - 30}
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleState {
    Idle(u32),
    Charging(u32, *mut Soldier),
    Attacking(u32, *mut Soldier),
    Defending(u32, *mut Soldier),
    Dead
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Team {
    Red,
    Blue
} impl Team {
    fn to_string(&self) -> String {
        match self {
            Team::Red => "Red".to_string(),
            Team::Blue => "Blue".to_string()
        }
    }
}