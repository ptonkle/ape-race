# The Spindle: pitfalls to avoid

The general list is MAPS.md "Common gotchas". This is what it means for this map.

- **Everything is placed in the flat frame** (x round the tube, y up from the floor toward the axis, z along it) and bent once: still scenery when it is built (`spindlescape::scenery` bends each chunk), moving shapes in the app's view (`spindlescape::bend`). Never place a shape in world coordinates except through `Warp::affine_of` (the space past the end).
- **A shape is turned, not stretched, by the bend.** Anything that must tile round the tube is narrowed for its height first (`Warp::narrow`, `spindlescape::tile`): at the axle a span is a fifth of its width on the floor.
- **The seam (x = ±C/2):** no vine within `SEAM_CLEAR` of it, no block across it (split it in two). Moving columns use `Warp::wrap_x` for every distance.
- **The ceiling is the axle's underside** (`AXLE_UNDER`, 35 m). Vines hang from just under it; the camera is kept 3 m under it (`spindlescape::camera`), or it would see through the axle or past the axis.
- **No side walls:** `half_width` is half the circumference and `Course::contain` keeps no corridor; the AI must be kept on the route by the vines, not by walls.
- **Blocks:** solid on their sides and underside, their tops floors (`Course::support`); slime's top bounces. A tall block in the route needs a way past it within reach of the vines (gotcha 17: no pocket an AI cannot leave).
- **Moving columns are a pure function of the tick** (`Spinner::x_at`).
- **Other maps unchanged:** every hook is behind `Course.spindle`; the map's own dice (`layout::DICE`).
