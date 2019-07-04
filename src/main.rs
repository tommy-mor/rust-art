use mini_gl_fb::glutin::{MouseButton, VirtualKeyCode};
use mini_gl_fb::{BufferFormat, Config};

use rand::prelude::*;
use std::time::SystemTime;


const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const LEFT: u8 = 0;
const RIGHT: u8 = 1;
const UP: u8 = 2;
const DOWN: u8 = 3;


struct TuringMachine {
    table: Vec<u8>,
    map: [u8; WIDTH * HEIGHT],
    num_states: u8,
    num_symbols: u8,
    rng: SmallRng,
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
        let mut table = Vec::with_capacity((num_states * num_symbols) as usize * 3);
        for _ in 0..(num_states * num_symbols * 3) {
            table.push(0);
        }
        let mut map = [0u8; WIDTH * HEIGHT];
        let mut rng = SmallRng::from_entropy();
        let mut state = 0;
        let mut xpos = 0;
        let mut ypos = 0;
        let mut itr_count = 0;

        TuringMachine {
            table,
            map,
            num_states,
            num_symbols,
            rng,
            state,
            xpos,
            ypos,
            itr_count,
        }
    }

    fn init(&mut self) {
        for st in 0..self.num_states {
            for sy in 0..self.num_symbols {
                let st1 = self.rng.gen_range(0, self.num_states);
                let sy1 = self.rng.gen_range(1, self.num_symbols);
                let ac = self.rng.gen_range(0, 4);
                self.set_trans(
                    st,
                    sy,
                    st1,
                    sy1,
                    ac,
                );
            }
        }
    }

    fn get_render_buf(&self) -> Vec<bool> {
        let mut r_vec = vec![false; WIDTH * HEIGHT];
        for i in 0..self.map.len() {
            r_vec[i] = self.map[i] == 0;
        }
        r_vec
    }

    fn set_trans(&mut self, st0: u8, sy0: u8, st1: u8, sy1: u8, ac: u8) {
        let idx = ((self.num_states * sy0 + st0) * 3) as usize;

        self.table[idx + 0] = st1;
        self.table[idx + 1] = sy1;
        self.table[idx + 2] = ac;
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
            let sadf = (WIDTH * self.ypos + self.xpos);

            let sy = self.map[sadf];
            let st = self.state;

            let idx: usize = (self.num_states * sy as u8 + st as u8) as usize;
            let st = self.table[idx + 0];
            let sy = self.table[idx + 1];
            let ac = self.table[idx + 2];

            self.state = st;

            self.map[sadf] = sy;

            match ac {
                LEFT=> {
                    self.xpos  += 1;
                    if self.xpos >= WIDTH  {
                        self.xpos -= WIDTH;
                    }
                },
                RIGHT => {
                    self.xpos = if let Some(x) = self.xpos.checked_sub(1) {
                        x
                    } else {
                        WIDTH-1
                    };
                },
                UP=> {
                    if self.ypos == 0 {
                        self.ypos += HEIGHT - 1;
                    }
                    self.ypos  -= 1;
                },
                DOWN => {
                    self.ypos += 1;
                    if self.ypos >= HEIGHT {
                        self.ypos -= HEIGHT;
                    }
                }
                _ => panic!("invalid action")

            }
            self.itr_count += 1;
        }
    }
}

fn main() {
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "gaymers",
        window_size: (WIDTH as _, HEIGHT as _),
        buffer_size: (WIDTH as _, HEIGHT as _),
        ..Default::default()
    });

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_post_process_shader(POST_PROCESS);


    let mut machine = TuringMachine::new(3,4);
    machine.init(); //very smart


    let mut neighbors = vec![0; WIDTH * HEIGHT];
    let mut cells = vec![false; WIDTH * HEIGHT];


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
//            cells[y * WIDTH + x] = true;
//            fb.update_buffer(&cells);
//            // Give the user extra time to make something pretty each time they click
//            previous = SystemTime::now();
//            extra_delay = (extra_delay + 0.5).min(2.0);
        }

        // Each generation should stay on screen for half a second
        if seconds > 0.01 + extra_delay {
            previous = SystemTime::now();
//            calculate_neighbors(&mut cells, &mut neighbors);
//            make_some_babies(&mut cells, &mut neighbors);
            machine.update(5000);
            fb.update_buffer(&machine.get_render_buf());
            extra_delay = 0.0;
        } else if input.resized {
            fb.redraw();
        }

        true
    });
}

fn calculate_neighbors(cells: &mut [bool], neighbors: &mut [u32]) {
    // a very basic GOL implementation; assumes outside the grid is dead
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut n = 0;

            // Above
            if y > 0 {
                let j = y - 1;
                if x > 0 && cells[j * WIDTH + x - 1] {
                    n += 1;
                }
                if cells[j * WIDTH + x] {
                    n += 1;
                }
                if x < (WIDTH - 1) && cells[j * WIDTH + x + 1] {
                    n += 1;
                }
            }

            // On the same line
            if x > 0 && cells[y * WIDTH + x - 1] {
                n += 1;
            }
            if x < (WIDTH - 1) && cells[y * WIDTH + x + 1] {
                n += 1;
            }

            // Below
            if y < (HEIGHT - 1) {
                let j = y + 1;
                if x > 0 && cells[j * WIDTH + x - 1] {
                    n += 1;
                }
                if cells[j * WIDTH + x] {
                    n += 1;
                }
                if x < (WIDTH - 1) && cells[j * WIDTH + x + 1] {
                    n += 1;
                }
            }

            let cell = y * WIDTH + x;
            neighbors[cell] = n;
        }
    }
}

fn make_some_babies(cells: &mut [bool], neighbors: &mut [u32]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cell = y * WIDTH + x;

            if !cells[cell] {
                // if this cell is dead
                if neighbors[cell] == 3 {
                    // and it has three neighbors...
                    cells[cell] = true;
                }
                // else it stays dead
                continue;
            }
            // the cell is alive

            if neighbors[cell] <= 1 {
                // die from under population
                cells[cell] = false;
            } else if neighbors[cell] > 3 {
                // die from over population
                cells[cell] = false;
            }
            // else: survive to the next generation
        }
    }
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
