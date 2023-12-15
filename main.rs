use notan::draw::*;
use notan::graphics::renderer;
use notan::math::{Mat4, Vec3};
use notan::prelude::*;
use obj::{load_obj, Obj, Vertex};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::exit;
use std::{cmp, path};
use std::{fs, string};

//
//Begin misc. definitions
//

//enemy move patterns
//simply move from left to right side of screen
fn left_to_right(e: &mut Entity) {
    let speed_x = e.speed_x.clone();
    let speed = speed_x;
    e.move_x(speed);
}

//simply move from right to left side of screen
fn right_to_left(e: &mut Entity) {
    let speed_x = e.speed_x.clone();
    let speed = speed_x;
    e.move_x(speed * -1.0);
}

//stay perfectly still
fn stay_still(e: &mut Entity) {}

//
//End misc. definitions
//

//
//Begin visual stuff
//

//vertex shader
//I have no idea how to make one of these on my own yet, so i'm using
//  the one used shown on the cube rendering example on the Notan git repo
//All I can tell so far is that it the shader needs a procedural macro
const VERT: ShaderSource = notan::vertex_shader! {
    r#"
    #version 450
    layout(location = 0) in vec4 a_position;
    layout(location = 1) in vec4 a_color;

    layout(location = 0) out vec4 v_color;

    layout(set = 0, binding = 0) uniform Locals {
        mat4 u_matrix;
    };

    void main() {
        v_color = a_color;
        gl_Position = u_matrix * a_position;
    }
    "#
};

//fragment shader
//see above comment
const FRAG: ShaderSource = notan::fragment_shader! {
    r#"
    #version 450
    precision mediump float;

    layout(location = 0) in vec4 v_color;
    layout(location = 0) out vec4 color;

    void main() {
        color = v_color;
    }
    "#
};

//
//End visual stuff
//

//
// Begin game entity definitions
//

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

#[derive(Clone)]
//contains data about 3D models, since the Obj package I'm using doesn't have everythin the way it needs to be in order for Notan to use it
struct ModelData {
    name: String,
    vertices: Vec<f32>,
    indices: Vec<u16>,
}

#[derive(Clone)]
//contains data used to draw 3D models
struct Draw {
    clear_options: ClearOptions,
    pipeline: Pipeline,
    model_view_projection: notan::math::Mat4,
    vertex_info: VertexInfo,
    uniform_buffer: Buffer,
}

//the total state of the game
#[derive(AppState, Clone)]
struct State {
    p1: Entity,
    entities: Vec<Entity>,                 //stores entities in game
    timer: f32,             //stores time. used for entity spawning, game speed regulation
    models: Vec<ModelData>, //stores models
    score: i32, //stores score: increased by surviving and defeating enemies. higher score increases difficulty
    wavetimer: f32, //timer to regulate spawning
    attack_patterns: Vec<fn(&mut Entity)>, //stores movement patterns for enemies
    draw: Draw, //stores all data needed by the draw functions
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
                        vertices: self.models[0].vertices.clone(),
                        indices: self.models[0].indices.clone(),
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

    fn spawn_cycle(&mut self, app: &mut App) {
        //go off every 10 seconds
        self.wavetimer += app.timer.delta_f32();
        if self.wavetimer >= 10.0 {
            print!("Spawning wave");

            //size of wave. todo: make it scale based on score
            let wave_size = 5;
            let mut i = 0;
            while i < wave_size {
                self.spawn_enemy();
                i += 1;
            }
            self.wavetimer = 0.0;
        }
    }

    //spawn an enemy
    fn spawn_enemy(&mut self) {
        //create a new entity and add it to the game state's entity list
        println!("Spawning enemy");
        let center_x = 100.0;
        let center_y = 100.0;
        self.create_entity(
            EntityType::Enemy,
            WeaponType::None,
            5,
            ShipDraw {
                vertices: self.models[1].vertices.clone(),
                indices: self.models[1].indices.clone(),
                center_x: (center_x),
                center_y: (center_y),
            },
            0.0,
            0.0,
            0.0,
        );
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
#[derive(Clone)]
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
        //self.shape.v1x += d;
        //self.shape.v2x += d;
        //self.shape.v3x += d;
    }
    fn move_y(&mut self, d: f32) {
        self.shape.center_y += d;
        //self.shape.v1y += d;
        //self.shape.v2y += d;
        //self.shape.v3y += d;
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
#[derive(Clone)]
struct ShipDraw {
    center_x: f32,
    center_y: f32,
    vertices: Vec<f32>,
    indices: Vec<u16>,
}

//
//End game entity definitions
//

//
//Begin central program stuff
//

//main function: calles everything else
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
fn setup(gfx: &mut Graphics) -> State {
    //create model view projection matrix
    let projection = Mat4::perspective_rh_gl(45.0, 4.0 / 3.0, 0.1, 100.0);
    let view = Mat4::look_at_rh(
        Vec3::new(4.0, 3.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),
    );

    //create vertex info
    let vertex_info = VertexInfo::new()
        .attr(0, VertexFormat::Float32x3)
        .attr(1, VertexFormat::Float32x4);
    //create depth stencil
    let depth_stencil = DepthStencil {
        write: true,
        compare: CompareMode::Less,
    };

    //create pipeline
    let pipe = gfx
        .create_pipeline()
        .from(&VERT, &FRAG)
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_stencil)
        .build()
        .unwrap();

    //create clear options
    let clear_options = ClearOptions {
        color: Some(Color::TRANSPARENT),
        depth: Some(1.0),
        stencil: None,
    };

    //create model view projection
    let mvp = Mat4::IDENTITY * projection * view;
    //create uniform buffer
    let uniform_buffer = gfx
        .create_uniform_buffer(0, "Locals")
        .with_data(&mvp)
        .build()
        .unwrap();

    //create game state
    let mut s = State {
        p1: Entity {
            id: 0,
            etype: EntityType::Player,
            wtype: WeaponType::PlayerBasic,
            shape: ShipDraw {
                center_x: 40.0,
                center_y: 30.0,
                vertices: Vec::new(),
                indices: Vec::new(),
            },
            health: 1,
            speed_x: 0.0,
            speed_y: 0.0,
            top_speed: 5.0,
            collision_damage: 1,
            is_tangible: true,
        },
        entities: Vec::new(),
        timer: 0.0,
        wavetimer: 0.0,
        models: Vec::new(),
        score: 0,
        attack_patterns: Vec::new(),
        draw: Draw {
            pipeline: pipe,
            clear_options: clear_options,
            model_view_projection: mvp,
            vertex_info: vertex_info,
            uniform_buffer: uniform_buffer,
        },
    };

    //load models from files into list
    let files = match std::fs::read_dir("./target/debug/assets/models") {
        Ok(file) => file,
        Err(error) => panic!("Directory could not be read: {:?}", error),
    };
    for f in files {
        let file = match f {
            Ok(m) => m,
            Err(error) => panic!("Error in reading directory: {:?}", error),
        };
        let a = match File::open(file.path()) {
            Ok(file) => file,
            Err(error) => panic!("File not opened: {:?}", error),
        };
        let ifile = BufReader::new(a);
        let ob: obj::Obj = match load_obj(ifile) {
            Ok(o) => o,
            Err(error) => panic!("Object could not be processed {:?}", error),
        };
        let mut vertices = Vec::new();
        for v in ob.vertices {
            for p in v.position {
                vertices.push(p);
            }
            for n in v.normal {
                vertices.push(n);
            }
        }
        let name = ob.name;
        println!("Object loaded: {:?}", name);
        match name {
            None => s.models.push(ModelData {
                name: "unnamed".to_string(),
                vertices: vertices,
                indices: ob.indices,
            }),
            Some(out) => s.models.push(ModelData {
                name: out.to_string(),
                vertices: vertices,
                indices: ob.indices,
            }),
        };
    }

    s.p1.shape.vertices = s.models[0].vertices.clone();
    s.p1.shape.indices = s.models[0].indices.clone();

    //add attack patterns into pattern list
    //patterns will be randomly chosen when spawning waves
    s.attack_patterns.push(left_to_right);
    s.attack_patterns.push(right_to_left);
    //return finished state
    s
}

//controls what happens as the game updates
fn update(app: &mut App, state: &mut State) {
    state.timer = app.timer.delta_f32();
    if state.timer >= (1.0 / 240.0) {
        if app.keyboard.was_pressed(KeyCode::Q) {
            println!("Number of models loaded: {}", state.models.len());
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
            state.bullet(state.p1.clone(), 1);
        }

        //move entity along y coordinate, then decay speed
        if state.p1.shape.center_y < 600.0 && state.p1.shape.center_y > 0.0 {
            state.p1.move_y(state.p1.speed_y);
        }
        if state.p1.speed_y > 0.0 {
            state.p1.speed_y -= state.p1.speed_y * 0.13;
        } else if state.p1.speed_y < 0.0 {
            state.p1.speed_y -= state.p1.speed_y * 0.13;
        }

        //move entity along x coordinate, then decay speed
        if state.p1.shape.center_x < 1000.0 && state.p1.shape.center_x > 0.0 {
            state.p1.move_x(state.p1.speed_x);
        }
        if state.p1.speed_x > 0.0 {
            state.p1.speed_x -= state.p1.speed_x * 0.13;
        } else if state.p1.speed_x < 0.0 {
            state.p1.speed_x -= state.p1.speed_x * 0.13;
        }

        //move non player entities
        let mut i: usize = 0;
        while i < state.entities.len() {
            let e = state.entities[i].clone();
            //do the move
            state.entities[i].move_x(e.speed_x);
            state.entities[i].move_y(e.speed_y);

            //destroy any entities that are out of bounds
            if state.entities[i].shape.center_x > 1200.0
                || state.entities[i].shape.center_x < -200.0
            {
                state.entities.remove(i);
                if i != 0 {
                    i -= 1 as usize;
                }
            } else if state.entities[i].shape.center_y > 800.0
                || state.entities[i].shape.center_y < -200.0
            {
                state.entities.remove(i);
                if i != 0 {
                    i -= 1 as usize;
                }
            }
            i += 1 as usize;
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

        state.spawn_cycle(app);
        //reset timer
        state.timer = 0.0;
    }
}

//puts all of the data onto the window
fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    let mut renderer = gfx.create_renderer();

    //Display Score
    let score_str = "Score: ".to_owned() + state.score.to_string().as_str();
    let font = gfx
        .create_font(include_bytes!("assets/font/Ubuntu-B.ttf"))
        .unwrap();
    draw.text(&font, score_str.as_str());

    draw.clear(Color::BLACK);
    renderer.begin(Some(state.draw.clear_options));
    renderer.set_pipeline(&state.draw.pipeline);
    for e in &state.entities[..] {
        //load data into array to create buffers
        //create buffers
        let vertex_buffer = gfx
            .create_vertex_buffer()
            .with_info(&state.draw.vertex_info)
            .with_data(&e.shape.vertices[..])
            .build()
            .unwrap();
        let indices = unsafe { e.shape.indices.align_to().1 };
        let index_buffer = gfx
            .create_index_buffer()
            .with_data(indices)
            .build()
            .unwrap();
        renderer.bind_buffers(&[&vertex_buffer, &index_buffer, &state.draw.uniform_buffer]);
        renderer.draw(0, indices.len() as i32);
    }
    renderer.end();
    //draw.triangle(
    //(state.p1.shape.v1x, state.p1.shape.v1y),
    //(state.p1.shape.v2x, state.p1.shape.v2y),
    //(state.p1.shape.v3x, state.p1.shape.v3y),
    //);
    //for e in state.entities.clone() {
    //     draw.triangle(
    //         (e.shape.v1x, e.shape.v1y),
    //        (e.shape.v2x, e.shape.v2y),
    //        (e.shape.v3x, e.shape.v3y),
    //    )
    //    .color(Color::AQUA);
    //}
    gfx.render(&renderer);
    gfx.render(&draw);
}
