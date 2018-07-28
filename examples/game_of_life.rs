extern crate mini_gl_fb;

use mini_gl_fb::{Config, BufferFormat};

use std::time::SystemTime;

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

fn main() {
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "PSA: Conway wants you to appreciate group theory instead",
        window_size: (800.0, 800.0),
        buffer_size: (WIDTH as _, HEIGHT as _),
        .. Default::default()
    });

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_post_process_shader(POST_PROCESS);

    let mut neighbors = vec![0; WIDTH * HEIGHT];
    let mut cells = vec![false; WIDTH * HEIGHT];

    cells[5 * WIDTH + 10] = true;
    cells[5 * WIDTH + 11] = true;
    cells[5 * WIDTH + 12] = true;

    let mut previous = SystemTime::now();

    while fb.is_running() {
        let elapsed = previous.elapsed().unwrap();
        let seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

        // Each generation should stay on screen for half a second
        if seconds > 0.5 {
            previous = SystemTime::now();
            calculate_neighbors(&mut cells, &mut neighbors);
            make_some_babies(&mut cells, &mut neighbors);
            // Vsync should be enabled by default (open an issue if you need it disabled)
            // Vsync should cause this to block so we don't run too often or too fast #robust
            fb.update_buffer(&cells);
        }
    }
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
            } else if neighbors[cell] >= 3 {
                // die from over population
                cells[cell] = false;
            }
            // else: survive to the next generation
        }
    }
}

const POST_PROCESS: &str = "
    bool on_grid_line(float pos) {
        if (fract(pos) < 0.2) {
            return true;
        } else {
            return false;
        }
    }

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
        vec2 grid_pos = uv * 200;
        if (on_grid_line(grid_pos.x) || on_grid_line(grid_pos.y)) {
            sample = max(sample - 0.4, vec3(0.0, 0.0, 0.0));
        }
        r_frag_color = vec4(sample, 1.0);
    }
";
