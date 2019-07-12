use mini_gl_fb::glutin::{MouseButton, VirtualKeyCode};
use mini_gl_fb::BufferFormat;

use rand::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use arrayvec::ArrayVec;
use screenshot_rs::screenshot_window;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

const NUM_MACHINES: usize = 1;
const STEPS_PER_FRAME: u32 = 10;
const STARTENERGY: u32 = 10;
const REPLICATIONCOST: u32 = 500;

#[derive(Clone)]
enum Action {
    Up,
    Down,
    Left,
    Right,
    Wait,
    Replicate,
}

impl Distribution<Action> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        match rng.gen_range(0, 6) {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            3 => Action::Right,
            4 => Action::Wait,
            _ => Action::Replicate,
        }
    }
}

#[derive(Clone)]
struct Transition {
    state: u8,
    symbol: u8,
    action: Action,
}

#[derive(Clone)]
struct TuringMachine {
    table: ArrayVec<[Transition; 4096]>,
    num_states: u16,
    num_symbols: u16,
    state: u8,
    energy: u32,
    xpos: usize,
    ypos: usize,
    itr_count: u32,
}

/*
N states, one start state
K symbols
4 actions (left, right up, down)
N x K -> N x K x A
*/

impl TuringMachine {
    fn new(num_states: u16, num_symbols: u16) -> TuringMachine {
        assert!(num_states >= 1, "must have at least 1 state");
        assert!(num_symbols >= 2, "must have at least 2 symbols");
        assert!(
            num_states * num_symbols <= 4096,
            "num_states * num_symbols <= 4096"
        );

        let mut table = ArrayVec::new();
        let mut rng = SmallRng::from_entropy();
        for i in 0..(num_states * num_symbols) {
            /*
            let mut state = 0;
            if rng.gen_range(0,num_states*num_symbols/100) != 0 {
                state = rng.gen_range(0, num_states) as u8;
            }
            */
            let state = rng.gen_range(0, num_states) as u8;
            let trans = Transition {
                state: state,
                symbol: rng.gen_range(0, num_symbols) as u8,
                action: rng.gen(),
            };

            table.push(trans);
        }

        TuringMachine {
            table,
            num_states,
            num_symbols,
            state: 0,
            energy: STARTENERGY,
            xpos: rng.gen_range(0, WIDTH),
            ypos: rng.gen_range(0, HEIGHT),
            itr_count: 0,
        }
    }

    fn from_string(transition_hash: &str) -> TuringMachine {
        let mut trans_table = transition_hash.split(",").map(|n| u8::from_str(n).expect("not parsable"));
        let num_states = trans_table.next().unwrap() as u16;
        let num_symbols = trans_table.next().unwrap() as u16;

        let mut table = ArrayVec::new();
        for _ in 0..(num_states * num_symbols) {
            let state = trans_table.next().unwrap();
            let symbol = trans_table.next().unwrap();

            let action = match trans_table.next().unwrap() {
                0 => Action::Left,
                1 => Action::Right,
                2 => Action::Up,
                3 => Action::Down,
                _ => panic!("no such action"),
            };

            let trans = Transition {
                state,
                symbol,
                action,
            };

            table.push(trans);
        }

        TuringMachine {
            table,
            num_states,
            num_symbols,
            state: 0,
            energy: STARTENERGY,
            xpos: 0,
            ypos: 0,
            itr_count: 0,
        }
    }

    fn reset(&mut self) {
        self.state = 0;
        self.energy = STARTENERGY;
        self.ypos = 0;
        self.xpos = 0;
        self.itr_count = 0;
    }

    fn update(&mut self, map: &mut [u8; WIDTH * HEIGHT], num_iters: u32, machines: &mut Vec<TuringMachine>) {
        for _ in 0..num_iters {

            self.energy -= 1;

            let symbol = &mut map[WIDTH * self.ypos + self.xpos];

            self.energy += (*symbol as u32)/10;

            let trans = &self.table[(self.num_states as u8 * (*symbol) + self.state) as usize];
            self.state = trans.state;

            *symbol = trans.symbol;

            self.itr_count += 1;

            match trans.action {
                Action::Left => {
                    self.xpos += 1;
                    if self.xpos >= WIDTH {
                        self.xpos -= WIDTH;
                    }
                }
                Action::Right => {
                    self.xpos = if let Some(x) = self.xpos.checked_sub(1) {
                        x
                    } else {
                        WIDTH - 1
                    };
                }
                Action::Up => {
                    self.ypos = if let Some(y) = self.ypos.checked_sub(1) {
                        y
                    } else {
                        HEIGHT - 1
                    };

                }
                Action::Down => {
                    self.ypos += 1;
                    if self.ypos >= HEIGHT {
                        self.ypos -= HEIGHT;
                    }
                }
                Action::Wait => {

                }
                Action::Replicate => {
                    if self.energy > REPLICATIONCOST {
                        //println!("REPLICATE");
                        //TODO Higher costs for higher table complexity? Maybe just per step?
                        //TODO ensure STARTENERGY < REPLICATIONCOST!, but also see what happens otherwise
                        self.energy -= REPLICATIONCOST;
                        let mut newmachine = self.clone();
                        newmachine.xpos = (newmachine.xpos + 1) % WIDTH;
                        newmachine.ypos = (newmachine.ypos + 1) % HEIGHT;
                        newmachine.energy = STARTENERGY;
                        machines.push(newmachine);
                    }
                }
            }
        }
    }
}

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("art", WIDTH as f64, HEIGHT as f64);

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_post_process_shader(COLOR_SYMBOLS);

    //let mut machine = TuringMachine::from_string("5,4,4,2,1,1,3,2,4,3,1,2,2,3,1,2,1,3,2,0,2,2,3,2,3,0,2,3,2,4,2,2,0,2,0,1,1,0,2,3,0,1,2,1,2,3,3,3,2,0,1,1,3,2,2,0,2,2,3,3,2,0");
    //let mut machine = TuringMachine::from_string("3,6,2,2,3,2,4,0,0,1,0,2,1,2,1,1,0,1,2,3,2,3,0,2,1,0,2,5,3,2,5,2,2,4,1,1,5,0,2,4,3,0,4,0,0,1,1,2,1,3,2,1,0,2,2,0");

    let mut previous = SystemTime::now();

    let mut playing = true;
    let mut space_pressed = false;
    let mut s_pressed = false;

    let mut map: [u8; WIDTH * HEIGHT] = [0u8; WIDTH * HEIGHT];

    let mut machines : Vec<TuringMachine> = vec![];

    let mut ITER: u64 = 0;

    fb.glutin_handle_basic_input(|fb, input| {
        let elapsed = previous.elapsed().unwrap();
        let seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        if input.key_is_down(VirtualKeyCode::R) {
            let mut rng = SmallRng::from_entropy();
            //machine.reset();
            //machine.state = rng.gen_range(0, machine.num_states) as u8;
        }

        if input.key_is_down(VirtualKeyCode::S) {
            if !s_pressed {
                screenshot_window(format!("screenshot-{:?}.png", previous.duration_since(UNIX_EPOCH).expect("time went backwards")));
                s_pressed = true;
            }
        } else {
            s_pressed = false
        }

        if input.mouse_is_down(MouseButton::Left) {
            playing = true;
            //machine.reset();
        }

        if input.mouse_is_down(MouseButton::Right) {
            playing = true;
            machines = vec![];
            previous = SystemTime::now();
        }

        if input.key_is_down(VirtualKeyCode::Space) {
            if !space_pressed {
                playing = !playing;
                space_pressed = true;
            }
        } else {
            space_pressed = false;
        }

        if (seconds > 0.00) && playing {
            previous = SystemTime::now();

            let mut newmachines : Vec<TuringMachine> = vec![];
            for machine in &mut machines {
                machine.update(&mut map, STEPS_PER_FRAME, &mut newmachines);
            }
            println!("{}", newmachines.len());
            //machines.extend(newmachines);
            for newmachine in newmachines {
                machines.push(newmachine);
            }
            machines.retain(|machine| machine.energy > 0);

            if machines.len() < NUM_MACHINES {
                for i in 0..NUM_MACHINES-machines.len() {
                    machines.push(TuringMachine::new(50,64));
                }
            }

            fb.update_buffer(&map[..]);
            println!("Frequency: {} Machines: {}", 1.0/seconds, machines.len());

            //if ITER % 100 == 0 {
            if true {
                for i in 0..WIDTH * HEIGHT {
                    if map[i] > 0 {
                        map[i] -= 1;
                    }
                }
            }

            ITER += 1;
        }

        true
    });
}

const COLOR_SYMBOLS: &str = r#"


    vec3 hsv2rgb(vec3 c)
    {
        vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
        vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
        return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
    }


    void main_image( out vec4 r_frag_color, in vec2 uv )
    {
        float red = texture(u_buffer, uv).r;
        if (red == 0) {
            r_frag_color = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            r_frag_color = vec4(hsv2rgb(vec3(red*4, 0.7, 1.0)), 1.0);
        }
    }
"#;
