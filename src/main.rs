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

const WIDTH: usize = 1024;
const HEIGHT: usize = 1025;

enum Action {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Action> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        match rng.gen_range(0, 4) {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            _ => Action::Right,
        }
    }
}

struct Transition {
    state: u8,
    symbol: u8,
    action: Action,
}

struct TuringMachine {
    table: ArrayVec<[Transition; 4096]>,
    map: [u8; WIDTH * HEIGHT],
    num_states: u16,
    num_symbols: u16,
    state: u8,
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
        for _ in 0..(num_states * num_symbols) {
            let trans = Transition {
                state: rng.gen_range(0, num_states) as u8,
                symbol: rng.gen_range(0, num_symbols) as u8,
                action: rng.gen(),
            };

            table.push(trans);
        }

        TuringMachine {
            table,
            map: [0u8; WIDTH * HEIGHT],
            num_states,
            num_symbols,
            state: 0,
            xpos: 0,
            ypos: 0,
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
            map: [0u8; WIDTH * HEIGHT],
            num_states,
            num_symbols,
            state: 0,
            xpos: 0,
            ypos: 0,
            itr_count: 0,
        }
    }

    fn reset(&mut self) {
        self.state = 0;
        self.ypos = 0;
        self.xpos = 0;
        self.itr_count = 0;

        self.map = [0u8; WIDTH * HEIGHT];
    }

    fn update(&mut self, num_iters: u32) {
        for _ in 0..num_iters {
            let symbol = &mut self.map[WIDTH * self.ypos + self.xpos];

            let trans = &self.table[(self.num_states as u8 * (*symbol) + self.state) as usize];
            self.state = trans.state;

            *symbol = trans.symbol;

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
            }
            self.itr_count += 1;
        }
    }
}

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("art", WIDTH as f64, HEIGHT as f64);

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_post_process_shader(COLOR_SYMBOLS);

    //let mut machine = TuringMachine::from_string("5,4,4,2,1,1,3,2,4,3,1,2,2,3,1,2,1,3,2,0,2,2,3,2,3,0,2,3,2,4,2,2,0,2,0,1,1,0,2,3,0,1,2,1,2,3,3,3,2,0,1,1,3,2,2,0,2,2,3,3,2,0");
    let mut machine = TuringMachine::from_string("3,6,2,2,3,2,4,0,0,1,0,2,1,2,1,1,0,1,2,3,2,3,0,2,1,0,2,5,3,2,5,2,2,4,1,1,5,0,2,4,3,0,4,0,0,1,1,2,1,3,2,1,0,2,2,0");

    let mut previous = SystemTime::now();

    let mut playing = true;
    let mut space_pressed = false;
    let mut s_pressed = false;

    fb.glutin_handle_basic_input(|fb, input| {
        let elapsed = previous.elapsed().unwrap();
        let seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        if input.key_is_down(VirtualKeyCode::R) {
            let mut rng = SmallRng::from_entropy();
            machine.reset();
            machine.state = rng.gen_range(0, machine.num_states) as u8;
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
            machine.reset();
        }

        if input.mouse_is_down(MouseButton::Right) {
            playing = true;
            machine = TuringMachine::new(12, 7);
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
            machine.update(50_000);
            fb.update_buffer(&machine.map[..]);
            println!("frequency {}", 1.0/seconds);
        }

        true
    });
}

const COLOR_SYMBOLS: &str = r#"
    void main_image( out vec4 r_frag_color, in vec2 uv )
    {
        int symbol = int(texture(u_buffer, uv).r * 255);
        switch (symbol) {
            case 0:
                // Red
                r_frag_color = vec4(255.0, 0.0, 0.0, 1.0);
                break;
            case 1:
                // Black
                r_frag_color = vec4(0.0, 0.0, 0.0, 1.0);
                break;
            case 2:
                // White
                r_frag_color = vec4(255.0, 255.0, 255.0, 1.0);
                break;
            case 3:
                // Green
                r_frag_color = vec4(0.0, 255.0, 0.0, 1.0);
                break;
            case 4:
                // Blue
                r_frag_color = vec4(0.0, 0.0, 255.0, 1.0);
                break;
            case 5:
                // Yellow
                r_frag_color = vec4(255.0, 255.0, 0.0, 1.0);
                break;
            case 6:
                // Magenta
                r_frag_color = vec4(255.0, 0.0, 255.0, 1.0);
                break;
        }
    }
"#;

