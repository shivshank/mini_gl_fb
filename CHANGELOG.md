# Change Log

## [v0.5.1] - 2018-08-25

Started a change log!

### Fixed

 - Vertical mouse position was wrong, v0.5.0 changed coordinate systems from a window-like one
   to a UV-like one (0, 0 is now the lower left) but the glutin basic input feature did not
   respect this and reported old style mouse positions
