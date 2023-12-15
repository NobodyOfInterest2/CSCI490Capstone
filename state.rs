mod state;

//the total state of the game
#[derive(AppState)]
struct State {
    p1: Entity,
    entities: Vec<Entity>, //stores entities in game
    timer: f32,            //stores time. used for entity spawning, game speed regulation
    models: Vec<Obj>,      //stores models
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
