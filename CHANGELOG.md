# Change Log

## [v0.6.0] - 2018-08-25

I considered adding `mouse_scale` to `BasicInput` which would allow the user to map back to
pixel coordinates if that's really what they want, but most of the time I'm guessing people
will want buffer coordinates.

The library needs to expose window size better (probably via `window_size` in `BasicInput`),
but I will hold off on both of these additions until there is a clear use case. It is not clear
what type (float or integer) these fields should be (logical size or physical size?).

A `mouse_as_buffer_index` method might also be useful for `BasicInput`.

### Changed

 - Changed glutin basic input handling to report positions as f64 instead of usize, which is a
   bit limiting.

## [v0.5.1] - 2018-08-25

Started a change log!

### Fixed

 - Vertical mouse position was wrong, v0.5.0 changed coordinate systems from a window-like one
   to a UV-like one (0, 0 is now the lower left) but the glutin basic input feature did not
   respect this and reported old style mouse positions
