use mini_gl_fb::glutin::{MouseButton, VirtualKeyCode};
use mini_gl_fb::{BufferFormat, Config};

use rand::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use std::time::SystemTime;

use arrayvec::ArrayVec;


const WIDTH: usize = 1024;
const HEIGHT: usize = 1024;

const LEFT: u8 = 0;
const RIGHT: u8 = 1;
const UP: u8 = 2;
const DOWN: u8 = 3;

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
    action: Action
}

const color_map: [u8; 24] = [
    255,0  ,0  ,    // Initial symbol color
    0  ,0  ,0  ,    // Black
    255,255,255,    // White
    0  ,255,0  ,    // Green
    0  ,0  ,255,    // Blue
    255,255,0  ,    // Yellow
    0  ,255,255,
    255,0  ,255,
];

struct TuringMachine {
    table: ArrayVec<[Transition; 4096]>,
    map: [u8; WIDTH * HEIGHT],
    num_states: u8,
    num_symbols: u8,
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
    fn new(num_states: u8, num_symbols: u8) -> TuringMachine {
        assert!(num_states >= 1, "must have at least 1 state");
        assert!(num_symbols >= 2, "must have at least 2 symbols");
        assert!((num_states as u16 * num_symbols as u16) <= 4096, "num_states * num_symbols <= 4096");

        let mut map = [0u8; WIDTH * HEIGHT];
        let mut state = 0;
        let mut xpos = 0;
        let mut ypos = 0;
        let mut itr_count = 0;

        let mut table = ArrayVec::new();
        let mut rng = SmallRng::from_entropy();
        for _ in 0..(num_states as u16 * num_symbols as u16) {
            let trans = Transition { 
                state: rng.gen_range(0, num_states),
                symbol: rng.gen_range(0, num_symbols),
                action: rng.gen(),
            };
            
            table.push(trans);
        }

        TuringMachine {
            table,
            map,
            num_states,
            num_symbols,
            state,
            xpos,
            ypos,
            itr_count,
        }
    }

    fn get_render_buf(&self) -> Vec<[u8;4]> {
        let mut r_vec = vec![[0u8, 255u8, 255u8, 255u8]; WIDTH * HEIGHT];
        for (sy, rv) in self.map.iter().zip(r_vec.iter_mut()) {
            let r = color_map[(3 * sy + 0) as usize];
            let g = color_map[(3 * sy + 1) as usize];
            let b = color_map[(3 * sy + 2) as usize];
            *rv = [r, g, b, 255];
        }
        r_vec
    }

    fn reset(&mut self) {
        self.state = 0;
        self.ypos = 0;
        self.xpos = 0;
        self.itr_count = 0;

        self.map = [0u8; WIDTH * HEIGHT];
    }

    fn update(&mut self, num_iters: u32) {
        for i in 0..num_iters {
            let symbol = &mut self.map[WIDTH * self.ypos + self.xpos];

            let trans = &self.table[(self.num_states * (*symbol) + self.state) as usize];
            self.state = trans.state;

            *symbol = trans.symbol;

            match trans.action {
                Action::Left => {
                    self.xpos  += 1;
                    if self.xpos >= WIDTH  {
                        self.xpos -= WIDTH;
                    }
                },
                Action::Right => {
                    self.xpos = if let Some(x) = self.xpos.checked_sub(1) {
                        x
                    } else {
                        WIDTH-1
                    };
                },
                Action::Up => {
                    self.ypos = if let Some(y) = self.ypos.checked_sub(1) {
                        y
                    } else {
                        HEIGHT-1
                    };
                },
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
    let mut fb = mini_gl_fb::gotta_go_fast("gaymers", WIDTH as f64, HEIGHT as f64);

    fb.change_buffer_format::<u8>(BufferFormat::RGBA);
    //fb.use_post_process_shader(POST_PROCESS);


    let mut machine = TuringMachine::new(3, 4);
    //machine.init(); //very smart

    let mut previous = SystemTime::now();
    let mut extra_delay: f64 = 0.0;

    fb.glutin_handle_basic_input(|fb, input| {
        let elapsed = previous.elapsed().unwrap();
        let seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        if input.mouse_is_down(MouseButton::Left) {
            // Mouse was pressed
            let (x, y) = input.mouse_pos;
            let x = x.min(WIDTH as f64 - 0.0001).max(0.0).floor() as usize;
            let y = y.min(HEIGHT as f64 - 0.0001).max(0.0).floor() as usize;

            machine.reset();
//            cells[y * WIDTH + x] = true;
//            fb.update_buffer(&cells);
//            // Give the user extra time to make something pretty each time they click
//            previous = SystemTime::now();
//            extra_delay = (extra_delay + 0.5).min(2.0);
        }

        if input.mouse_is_down(MouseButton::Right) {
            // Mouse was pressed
            let (x, y) = input.mouse_pos;
            let x = x.min(WIDTH as f64 - 0.0001).max(0.0).floor() as usize;
            let y = y.min(HEIGHT as f64 - 0.0001).max(0.0).floor() as usize;

            machine = TuringMachine::new(3,4);
        //    machine.init();
//            cells[y * WIDTH + x] = true;
//            fb.update_buffer(&cells);
//            // Give the user extra time to make something pretty each time they click
            previous = SystemTime::now();
//            extra_delay = (extra_delay + 0.5).min(2.0);
        }

        // Each generation should stay on screen for half a second
        if seconds > 0.00 + extra_delay {
            previous = SystemTime::now();
//            calculate_neighbors(&mut cells, &mut neighbors);
//            make_some_babies(&mut cells, &mut neighbors);
            machine.update(500000);
            fb.update_buffer(&machine.get_render_buf());
            extra_delay = 0.0;
            println!("frequency {}", 1.0/seconds);
        } else if input.resized {
            fb.redraw();
        }

        true
    });
}

const POST_PROCESS: &str = r#"

    void main_image( out vec4 r_frag_color, in vec2 uv )
    {
        // A bool is stored as 1 in our image buffer
        // OpenGL will map that u8/bool onto the range [0, 1]
        // so the u8 1 in the buffer will become 1 / 255 or 0.0
        // multiply by 255 to turn 1 / 255 into full intensity and leave 0 as 0
        vec3 sample = texture(u_buffer, uv).rrr * 255.0;
        // invert it since that's how GOL stuff is typically shown
        sample = 1.0 - sample;
        // attempt to add some grid lines (assumes width and height of image are 200)...

        r_frag_color = vec4(sample, 1.0);
    }
"#;
