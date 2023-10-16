use notan::draw::*;
use notan::prelude::*;
use std::cmp;

//defines each entity type that will be used
#[derive(Clone, Copy)]
enum EntityType {
    Player, //the entity that the player controls. can collide with enemies and their projectiles
    PlayerProjectile, //any projectile that the player entity spawns. can collide with enemies
    Enemy, //any non-player, non-projectile entity. can collide with the player and their projectiles
    EnemyProjectile, //projectiles spawned by enemies. can collide with the player
    Object, //objects that are none of the above. collides with everything
    Effect, //miscellaneous effects. collides with everything but projectiles
}

//defines each weapon
#[derive(Clone, Copy)]
enum WeaponType {
    None,        //no weapon. some entities are unarmed
    PlayerBasic, //default weapon used for testing
}

//defines each general state that the game can be in
enum GameState {
    Combat,
    Paused,
    Title,
    Settings,
}

//the total state of the game
#[derive(AppState)]
struct State {
    p1: Entity,
    entities: Vec<Entity>,
}

//general functions used by the game state
impl State {
    //spawn a projectile moving with a given x and y speed
    fn bullet(&mut self, e: Entity, punch_through: i32) {
        //create projectile and add it to the game state's entity list
        //can't think of any way to do this without simply checking each one one by one
        match e.wtype {
            //player weapons
            WeaponType::PlayerBasic => {
                println!(
                    "Space pressed. Firing weapon at ({}, {})",
                    e.shape.center_x, e.shape.center_y
                );
                self.create_entity(
                    EntityType::PlayerProjectile,
                    WeaponType::None,
                    punch_through,
                    ShipDraw {
                        v1x: (e.shape.center_x),
                        v1y: (e.shape.center_y + 20.0),
                        v2x: (e.shape.center_x - 30.0),
                        v2y: (e.shape.center_y + 20.0),
                        v3x: (e.shape.center_x + 30.0),
                        v3y: (e.shape.center_y - 20.0),
                        center_x: (e.shape.center_x),
                        center_y: (e.shape.center_y),
                    },
                    0.0,
                    -1.0,
                    5.0,
                );
            }
            //non player weapons
            WeaponType::None => {
                /*do nothing*/
                println!("Entity {} is unarmed.", e.id);
            }
        }
    }

    //spawn an enemy
    fn spawn_enemy(&mut self) {
        //create a new entity and add it to the game state's entity list
    }

    //entity creation. the spawning functions will call this to actually generate the entity
    fn create_entity(
        &mut self,
        entype: EntityType,
        weptype: WeaponType,
        health: i32,
        enshape: ShipDraw,
        xspeed: f32,
        yspeed: f32,
        topspeed: f32,
    ) -> i32 {
        //sort the entity list
        self.entities.sort_by_key(|en| en.id);
        let mut id = 0;
        //loop through the sorted list, and set id to the first integer that does not show up in the list
        for e in &self.entities {
            if id == e.id {
                //for each id that is in the list, go to the next
                id = id + 1;
            } else {
                //if an id is not present, use that one
                break;
            }
        }
        println!("Creating new entity with id of {}.", id);
        self.entities.push(Entity {
            id: id,
            etype: entype,
            wtype: weptype,
            shape: enshape,
            health: health,
            speed_x: xspeed,
            speed_y: yspeed,
            top_speed: topspeed,
            collision_damage: 1,
            is_tangible: true,
        });
        println!("Now there are {} entities.", self.entities.len());
        id
    }

    //remove an entity after it meets its conditions to be removed
    //ex: a projectile leaves the screen, an object loses its health, etc.
    fn despawn(&mut self, id: i32) {
        //find the id of the object and remove it from the vector
        for e in &self.entities {
            if e.id == id {
                println!("deleting entity with id {}", id);
            }
        }
    }

    //fire the player's weapon
    //check what pattern projectiles need to be made in, create that patters
    //todo: implement fire rate throttle
    // the weapon needs to have some sort of cooldown or it will simply fire every frame. patterns with more projectiles may cause slowdowns if not careful
    fn fire_weapon(&mut self) {}
}

//entities: anythign that is not UI nor background
#[derive(Clone, Copy)]
struct Entity {
    //the identification of the entity in the entity list
    id: i32,
    //the classification of the entity. determines various behaviors
    etype: EntityType,
    //the equipped weapon. detemines what pattern is created when pressing the fire key
    wtype: WeaponType,
    //the graphical data of the entity
    shape: ShipDraw,
    //how much damage an entity can recieve without being destroyed
    health: i32,
    //current speed
    speed_x: f32,
    speed_y: f32,
    //speed cannot exceed this value
    top_speed: f32,
    //damage for if two objects collide. projectiles are immune to this
    collision_damage: i32,
    //if true, ignore all collision regardless of class
    is_tangible: bool,
}

//functions called by entities
impl Entity {
    //move the entity along the screen
    fn move_x(&mut self, d: f32) {
        self.shape.center_x += d;
        self.shape.v1x += d;
        self.shape.v2x += d;
        self.shape.v3x += d;
    }
    fn move_y(&mut self, d: f32) {
        self.shape.center_y += d;
        self.shape.v1y += d;
        self.shape.v2y += d;
        self.shape.v3y += d;
    }

    //subtract health from an entity, removing it if its health reaches zero
    //projectiles will use this to determine if they should be able to hit multiple entities, and if so how many
    fn damage(&mut self, damage: i32) {
        //subract the damage from the health of the entity
        self.health -= damage;

        //remove if health is less than or equal to zero
        if self.health <= 0 {
            //remove
        }
    }
}

//stores data about what to draw
#[derive(Clone, Copy)]
struct ShipDraw {
    v1x: f32,
    v1y: f32,
    v2x: f32,
    v2y: f32,
    v3x: f32,
    v3y: f32,
    center_x: f32,
    center_y: f32,
}

fn main() -> Result<(), String> {
    println!("Hello World!");
    let window_config = WindowConfig::new().set_title("Test").set_size(1000, 600);
    notan::init_with(setup)
        .add_config(window_config)
        .update(update)
        .draw(draw)
        .add_config(DrawConfig)
        .build()
}

//sets things up before everything starts
fn setup() -> State {
    State {
        p1: Entity {
            id: 0,
            etype: EntityType::Player,
            wtype: WeaponType::PlayerBasic,
            shape: ShipDraw {
                center_x: 40.0,
                center_y: 30.0,
                //right now, start with
                //(40,10)
                v1x: 40.0,
                v1y: 10.0,
                //(10,50)
                v2x: 10.0,
                v2y: 50.0,
                //(70,50)
                v3x: 70.0,
                v3y: 50.0,
            },
            health: 1,
            speed_x: 0.0,
            speed_y: 0.0,
            top_speed: 5.0,
            collision_damage: 1,
            is_tangible: true,
        },
        entities: Vec::new(),
    }
}

//controls what happens as the game updates
fn update(app: &mut App, state: &mut State) {
    if app.keyboard.was_pressed(KeyCode::Q) {
        app.exit();
    }
    if app.keyboard.is_down(KeyCode::W) {
        if state.p1.speed_y > (-1.0 * state.p1.top_speed) {
            state.p1.speed_y += -5.0;
        }
    }
    if app.keyboard.is_down(KeyCode::S) {
        if state.p1.speed_y < (1.0 * state.p1.top_speed) {
            state.p1.speed_y += 5.0;
        }
    }
    if app.keyboard.is_down(KeyCode::A) {
        if state.p1.speed_x > (-1.0 * state.p1.top_speed) {
            state.p1.speed_x += -5.0;
        }
    }
    if app.keyboard.is_down(KeyCode::D) {
        if state.p1.speed_x < (1.0 * state.p1.top_speed) {
            state.p1.speed_x += 5.0;
        }
    }

    //fire player weapon
    if app.keyboard.is_down(KeyCode::Space) {
        println!("firing weapon!");
        state.bullet(state.p1, 1);
    }

    //move entity along y coordinate, then decay speed
    if state.p1.shape.center_y < 600.0 && state.p1.shape.center_y > 0.0 {
        state.p1.move_y(state.p1.speed_y);
    }
    if state.p1.speed_y > 0.0 {
        state.p1.speed_y -= state.p1.speed_y * 0.25;
    } else if state.p1.speed_y < 0.0 {
        state.p1.speed_y += state.p1.speed_y * 0.25;
    }

    //move entity along x coordinate, then decay speed
    if state.p1.shape.center_x < 1000.0 && state.p1.shape.center_x > 0.0 {
        state.p1.move_x(state.p1.speed_x);
    }
    if state.p1.speed_x > 0.0 {
        state.p1.speed_x -= state.p1.speed_x * 0.25;
    } else if state.p1.speed_x < 0.0 {
        state.p1.speed_x += state.p1.speed_x * 0.25;
    }

    //move non player entities
    for i in 0..(state.entities.len()) {
        let e = state.entities[i];

        //do the move
        state.entities[i].move_x(e.speed_x);
        state.entities[i].move_y(e.speed_y);

        //destroy any entities that are out of bounds
        if e.shape.center_x > 1200.0 || e.shape.center_x < -200.0 {
            state.despawn(e.id);
        } else if e.shape.center_y > 800.0 || e.shape.center_y < -200.0 {
            state.despawn(e.id);
        }
    }

    //check collison
    //the most efficient way that I could find at eh moment is iterating through the list in a nested loop
    //since each pair needs to only be checked once, some comparisons can be skipped
    //it does slow down at ~1k entities
    //ex: if a touches b, then by definition b must also touch a
    let l = state.entities.len();
    //for each entity
    for i in 0..l {
        let m = l - i;
        //for each entity after entity i
        for j in 0..m {
            if false
            //touches(state.entities[i], state.entities[m])
            {
                //touch(state.entities[i], state.entities[m]);
            }
        }
    }

    //if the player is out of bounds, push them back in bounds and stop them
    //y
    //positive
    if state.p1.shape.center_y > 600.0 {
        state.p1.speed_y = 0.0;
        state.p1.move_y(-state.p1.top_speed);
    }

    //negative
    if state.p1.shape.center_y < 0.0 {
        state.p1.speed_y = 0.0;
        state.p1.move_y(state.p1.top_speed);
    }

    //x
    //positive
    if state.p1.shape.center_x > 1000.0 {
        state.p1.speed_x = 0.0;
        state.p1.move_x(-state.p1.top_speed);
    }

    //negative
    if state.p1.shape.center_x < 0.0 {
        state.p1.speed_x = 0.0;
        state.p1.move_x(state.p1.top_speed);
    }
}

//puts all of the data onto the window
fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    draw.triangle(
        (state.p1.shape.v1x, state.p1.shape.v1y),
        (state.p1.shape.v2x, state.p1.shape.v2y),
        (state.p1.shape.v3x, state.p1.shape.v3y),
    );
    for e in state.entities.clone() {
        draw.triangle(
            (e.shape.v1x, e.shape.v1y),
            (e.shape.v2x, e.shape.v2y),
            (e.shape.v3x, e.shape.v3y),
        )
        .color(Color::AQUA);
    }
    gfx.render(&draw);
}
