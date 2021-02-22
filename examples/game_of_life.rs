#[macro_use]
extern crate mini_gl_fb;

use mini_gl_fb::BufferFormat;
use mini_gl_fb::glutin::event::{VirtualKeyCode, MouseButton};
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::dpi::LogicalSize;

use std::time::{Instant, Duration};
use mini_gl_fb::breakout::Wakeup;

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const NORMAL_SPEED: u64 = 500;
const TURBO_SPEED: u64 = 20;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut fb = mini_gl_fb::get_fancy(config! {
        window_title: String::from("PSA: Conway wants you to appreciate group theory instead"),
        window_size: LogicalSize::new(800.0, 800.0),
        buffer_size: Some(LogicalSize::new(WIDTH as _, HEIGHT as _))
    }, &event_loop);

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_post_process_shader(POST_PROCESS);

    let mut neighbors = vec![0; WIDTH * HEIGHT];
    let mut cells = vec![false; WIDTH * HEIGHT];

    cells[5 * WIDTH + 10] = true;
    cells[5 * WIDTH + 11] = true;
    cells[5 * WIDTH + 12] = true;

    cells[50 * WIDTH + 50] = true;
    cells[51 * WIDTH + 51] = true;
    cells[52 * WIDTH + 49] = true;
    cells[52 * WIDTH + 50] = true;
    cells[52 * WIDTH + 51] = true;

    // ID of the Wakeup which means we should update the board
    let mut update_id: Option<u32> = None;

    fb.glutin_handle_basic_input(&mut event_loop, |fb, input| {
        // We're going to use wakeups to update the grid
        input.wait = true;

        if update_id.is_none() {
            update_id = Some(input.schedule_wakeup(Instant::now() + Duration::from_millis(500)))
        } else if let Some(mut wakeup) = input.wakeup {
            if Some(wakeup.id) == update_id {
                // Time to update our grid
                calculate_neighbors(&mut cells, &mut neighbors);
                make_some_babies(&mut cells, &mut neighbors);
                fb.update_buffer(&cells);

                // Reschedule another update
                wakeup.when = Instant::now() + Duration::from_millis(
                    if input.key_is_down(VirtualKeyCode::LShift) {
                        TURBO_SPEED
                    } else {
                        NORMAL_SPEED
                    }
                );

                input.reschedule_wakeup(wakeup);
            }

            // We will get called again after all wakeups are handled
            return true;
        }

        if input.key_is_down(VirtualKeyCode::Escape) {
            return false;
        }

        if input.mouse_is_down(MouseButton::Left) || input.mouse_is_down(MouseButton::Right) {
            // Mouse was pressed
            let (x, y) = input.mouse_pos;
            let x = x.min(WIDTH as f64 - 0.0001).max(0.0).floor() as usize;
            let y = y.min(HEIGHT as f64 - 0.0001).max(0.0).floor() as usize;
            cells[y * WIDTH + x] = input.mouse_is_down(MouseButton::Left);
            fb.update_buffer(&cells);
            // Give the user extra time to make something pretty each time they click
            if !input.key_is_down(VirtualKeyCode::LShift) {
                input.adjust_wakeup(update_id.unwrap(), Wakeup::after_millis(2000));
            }
        }

        // TODO support right shift. Probably by querying modifiers somehow. (modifiers support)
        if input.key_pressed(VirtualKeyCode::LShift) {
            // immediately update
            input.adjust_wakeup(update_id.unwrap(), Wakeup::after_millis(0));
        } else if input.key_released(VirtualKeyCode::LShift) {
            // immediately stop updating
            input.adjust_wakeup(update_id.unwrap(), Wakeup::after_millis(NORMAL_SPEED));
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
