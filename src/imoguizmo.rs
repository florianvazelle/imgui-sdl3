use glam::{Mat4, Vec2, Vec3, Vec4};
use imgui::{DrawListMut, ImColor32, MouseButton, Ui};
use std::cmp::Ordering;
use std::f32;
use std::sync::LazyLock;

pub mod internal {
    use std::cell::RefCell;

    #[derive(Debug, Clone, Copy)]
    pub struct RectConfig {
        pub m_x: f32,
        pub m_y: f32,
        pub m_size: f32,
    }

    impl Default for RectConfig {
        fn default() -> Self {
            Self {
                m_x: 0.0,
                m_y: 0.0,
                m_size: 100.0,
            }
        }
    }

    thread_local! {
        pub static RECT: RefCell<RectConfig> = RefCell::new(RectConfig::default());
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DragState {
        pub active: bool,
        pub last_mouse: [f32; 2],
        pub yaw: f32,
        pub pitch: f32,
    }

    impl Default for DragState {
        fn default() -> Self {
            Self {
                active: false,
                last_mouse: [0.0, 0.0],
                yaw: 0.0,
                pitch: 0.0,
            }
        }
    }

    thread_local! {
        pub static DRAG_STATE: RefCell<DragState> = RefCell::new(DragState::default());
    }
}

#[inline]
pub fn check_inside_circle(center: Vec2, radius: f32, point: Vec2) -> bool {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    dx * dx + dy * dy <= radius * radius
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn draw_positive_line(
    ui: &Ui,
    draw_list: &mut DrawListMut,
    center: Vec2,
    axis: Vec2,
    color: ImColor32,
    radius: f32,
    thickness: f32,
    text: &str,
    selected: bool,
) {
    let line_end = Vec2 {
        x: center.x + axis.x,
        y: center.y + axis.y,
    };
    draw_list.add_line(center, line_end, color).thickness(thickness).build();
    draw_list.add_circle(line_end, radius, color).filled(true).build();

    // text size from ImGui
    let label_size = imgui::Ui::calc_text_size(ui, text);
    let text_pos = Vec2 {
        x: (line_end.x - 0.5 * label_size[0]).floor(),
        y: (line_end.y - 0.5 * label_size[1]).floor(),
    };
    if selected {
        draw_list.add_circle(line_end, radius, ImColor32::WHITE).build();
        draw_list.add_text(text_pos, ImColor32::WHITE, text);
    } else {
        draw_list.add_text(text_pos, ImColor32::BLACK, text);
    }
}

#[inline]
fn draw_negative_line(
    draw_list: &mut DrawListMut,
    center: Vec2,
    axis: Vec2,
    color: ImColor32,
    radius: f32,
    selected: bool,
) {
    let line_end = Vec2 {
        x: center.x - axis.x,
        y: center.y - axis.y,
    };
    draw_list.add_circle(line_end, radius, color).filled(true).build();
    if selected {
        draw_list.add_circle(line_end, radius, ImColor32::WHITE).build();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub line_thickness_scale: f32,
    pub axis_length_scale: f32,
    pub positive_radius_scale: f32,
    pub negative_radius_scale: f32,
    pub hover_circle_radius_scale: f32,
    pub x_circle_front_color: ImColor32,
    pub x_circle_back_color: ImColor32,
    pub y_circle_front_color: ImColor32,
    pub y_circle_back_color: ImColor32,
    pub z_circle_front_color: ImColor32,
    pub z_circle_back_color: ImColor32,
    pub hover_circle_color: ImColor32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            line_thickness_scale: 0.017,
            axis_length_scale: 0.33,
            positive_radius_scale: 0.075,
            negative_radius_scale: 0.05,
            hover_circle_radius_scale: 0.88,
            x_circle_front_color: ImColor32::from_rgba(255, 54, 83, 255),
            x_circle_back_color: ImColor32::from_rgba(154, 57, 71, 255),
            y_circle_front_color: ImColor32::from_rgba(138, 219, 0, 255),
            y_circle_back_color: ImColor32::from_rgba(98, 138, 34, 255),
            z_circle_front_color: ImColor32::from_rgba(44, 143, 255, 255),
            z_circle_back_color: ImColor32::from_rgba(52, 100, 154, 255),
            hover_circle_color: ImColor32::from_rgba(100, 100, 100, 130),
        }
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::default);

// Set the square rect where the gizmo will be drawn (top-left x,y and size)
pub fn set_rect(x: f32, y: f32, size: f32) {
    internal::RECT.with(|r| {
        let mut r = r.borrow_mut();
        r.m_x = x;
        r.m_y = y;
        r.m_size = size;
    });
}

// Positions an invisible window for the gizmo.
pub fn begin_frame<R, F: FnOnce() -> R>(ui: &Ui, background: bool, f: F) -> Option<R> {
    use imgui::{Condition, WindowFlags};

    let (pos, size) = internal::RECT.with(|r| {
        let r = *r.borrow();
        ([r.m_x, r.m_y], r.m_size)
    });

    let mut window = ui.window("imoguizmo");
    window = window
        .position(pos, Condition::Always)
        .size([size, size], Condition::Always)
        .flags(
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_INPUTS
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_FOCUS_ON_APPEARING
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                | if !background {
                    WindowFlags::NO_BACKGROUND
                } else {
                    WindowFlags::empty()
                },
        );

    window.build(f)
}

pub fn draw_gizmo(
    ui: &Ui,
    view_matrix: mint::ColumnMatrix4<f32>,
    projection_matrix: mint::ColumnMatrix4<f32>,
    pivot_distance: f32,
) -> Option<mint::ColumnMatrix4<f32>> {
    let mut draw_list = ui.get_window_draw_list();

    let (center, size, hsize) = internal::RECT.with(|r| {
        let r = *r.borrow();
        let h = r.m_size * 0.5;
        (
            Vec2 {
                x: r.m_x + h,
                y: r.m_y + h,
            },
            r.m_size,
            h,
        )
    });

    let mut view_projection: Mat4 = Mat4::from(projection_matrix) * Mat4::from(view_matrix);

    // Non-square aspect ratio correction (scale X components of x/z axes)
    {
        let p: Mat4 = Mat4::from(projection_matrix);
        let aspect_ratio = p.y_axis.y / p.x_axis.x;
        view_projection.x_axis.x *= aspect_ratio;
        view_projection.z_axis.x *= aspect_ratio;
    }

    let axis_length = size * CONFIG.axis_length_scale;
    let x_axis = view_projection * Vec4::new(axis_length, 0.0, 0.0, 0.0);
    let y_axis = view_projection * Vec4::new(0.0, axis_length, 0.0, 0.0);
    let z_axis = view_projection * Vec4::new(0.0, 0.0, axis_length, 0.0);

    let interactive = pivot_distance > 0.0;
    let io = ui.io();
    let mouse_pos = Vec2 {
        x: io.mouse_pos[0],
        y: io.mouse_pos[1],
    };

    let hover_circle_radius = hsize * CONFIG.hover_circle_radius_scale;
    if CONFIG.hover_circle_color != ImColor32::BLACK
        && interactive
        && check_inside_circle(center, hover_circle_radius, mouse_pos)
    {
        draw_list
            .add_circle(center, hover_circle_radius, CONFIG.hover_circle_color)
            .filled(true)
            .build();
    }

    let positive_radius = size * CONFIG.positive_radius_scale;
    let negative_radius = size * CONFIG.negative_radius_scale;
    let x_positive_closer = 0.0 >= x_axis.w;
    let y_positive_closer = 0.0 >= y_axis.w;
    let z_positive_closer = 0.0 >= z_axis.w;

    let mut pairs: Vec<(i32, f32)> = vec![
        (0, x_axis.w),
        (1, y_axis.w),
        (2, z_axis.w),
        (3, -x_axis.w),
        (4, -y_axis.w),
        (5, -z_axis.w),
    ];
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let mut selection: i32 = -1;
    if interactive {
        for &(idx, _) in pairs.iter().rev() {
            let hit = match idx {
                0 => check_inside_circle(
                    Vec2 {
                        x: center.x + x_axis.x,
                        y: center.y - x_axis.y,
                    },
                    positive_radius,
                    mouse_pos,
                ),
                1 => check_inside_circle(
                    Vec2 {
                        x: center.x + y_axis.x,
                        y: center.y - y_axis.y,
                    },
                    positive_radius,
                    mouse_pos,
                ),
                2 => check_inside_circle(
                    Vec2 {
                        x: center.x + z_axis.x,
                        y: center.y - z_axis.y,
                    },
                    positive_radius,
                    mouse_pos,
                ),
                3 => check_inside_circle(
                    Vec2 {
                        x: center.x - x_axis.x,
                        y: center.y + x_axis.y,
                    },
                    negative_radius,
                    mouse_pos,
                ),
                4 => check_inside_circle(
                    Vec2 {
                        x: center.x - y_axis.x,
                        y: center.y + y_axis.y,
                    },
                    negative_radius,
                    mouse_pos,
                ),
                5 => check_inside_circle(
                    Vec2 {
                        x: center.x - z_axis.x,
                        y: center.y + z_axis.y,
                    },
                    negative_radius,
                    mouse_pos,
                ),
                _ => false,
            };
            if hit {
                selection = idx;
                break;
            }
        }
    }

    let line_thickness = size * CONFIG.line_thickness_scale;
    for &(fst, _) in &pairs {
        match fst {
            0 => draw_positive_line(
                ui,
                &mut draw_list,
                center,
                Vec2 {
                    x: x_axis.x,
                    y: -x_axis.y,
                },
                if x_positive_closer {
                    CONFIG.x_circle_front_color
                } else {
                    CONFIG.x_circle_back_color
                },
                positive_radius,
                line_thickness,
                "X",
                selection == 0,
            ),
            1 => draw_positive_line(
                ui,
                &mut draw_list,
                center,
                Vec2 {
                    x: y_axis.x,
                    y: -y_axis.y,
                },
                if y_positive_closer {
                    CONFIG.y_circle_front_color
                } else {
                    CONFIG.y_circle_back_color
                },
                positive_radius,
                line_thickness,
                "Y",
                selection == 1,
            ),
            2 => draw_positive_line(
                ui,
                &mut draw_list,
                center,
                Vec2 {
                    x: z_axis.x,
                    y: -z_axis.y,
                },
                if z_positive_closer {
                    CONFIG.z_circle_front_color
                } else {
                    CONFIG.z_circle_back_color
                },
                positive_radius,
                line_thickness,
                "Z",
                selection == 2,
            ),
            3 => draw_negative_line(
                &mut draw_list,
                center,
                Vec2 {
                    x: x_axis.x,
                    y: -x_axis.y,
                },
                if !x_positive_closer {
                    CONFIG.x_circle_front_color
                } else {
                    CONFIG.x_circle_back_color
                },
                negative_radius,
                selection == 3,
            ),
            4 => draw_negative_line(
                &mut draw_list,
                center,
                Vec2 {
                    x: y_axis.x,
                    y: -y_axis.y,
                },
                if !y_positive_closer {
                    CONFIG.y_circle_front_color
                } else {
                    CONFIG.y_circle_back_color
                },
                negative_radius,
                selection == 4,
            ),
            5 => draw_negative_line(
                &mut draw_list,
                center,
                Vec2 {
                    x: z_axis.x,
                    y: -z_axis.y,
                },
                if !z_positive_closer {
                    CONFIG.z_circle_front_color
                } else {
                    CONFIG.z_circle_back_color
                },
                negative_radius,
                selection == 5,
            ),
            _ => {}
        }
    }

    if selection != -1 && ui.is_mouse_clicked(MouseButton::Left) {
        let model: Mat4 = Mat4::from(view_matrix).inverse();

        let pos = Vec3::new(model.w_axis.x, model.w_axis.y, model.w_axis.z);
        let z_axis_model = Vec3::new(model.z_axis.x, model.z_axis.y, model.z_axis.z);
        let pivot_pos = pos - (z_axis_model * pivot_distance);

        let ups = [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];

        let new_view = match selection {
            0 => Mat4::look_at_lh(pivot_pos + Vec3::new(pivot_distance, 0.0, 0.0), pivot_pos, ups[0]),
            1 => Mat4::look_at_lh(pivot_pos + Vec3::new(0.0, pivot_distance, 0.0), pivot_pos, ups[1]),
            2 => Mat4::look_at_lh(pivot_pos + Vec3::new(0.0, 0.0, pivot_distance), pivot_pos, ups[2]),
            3 => Mat4::look_at_lh(pivot_pos - Vec3::new(pivot_distance, 0.0, 0.0), pivot_pos, ups[3]),
            4 => Mat4::look_at_lh(pivot_pos - Vec3::new(0.0, pivot_distance, 0.0), pivot_pos, ups[4]),
            5 => Mat4::look_at_lh(pivot_pos - Vec3::new(0.0, 0.0, pivot_distance), pivot_pos, ups[5]),
            _ => return None,
        };

        return Some(mint::ColumnMatrix4::from(new_view));
    }

    let mut view_out: Option<mint::ColumnMatrix4<f32>> = None;

    let hover_inside = check_inside_circle(center, hover_circle_radius, mouse_pos);
    internal::DRAG_STATE.with(|state| {
        let mut s = state.borrow_mut();

        if ui.is_mouse_dragging(MouseButton::Left) {
            if !s.active && hover_inside {
                // Start drag
                s.active = true;
                s.last_mouse = [mouse_pos.x, mouse_pos.y];

                // let inv_view = Mat4::from(view_matrix).inverse();
                // let forward = -inv_view.z_axis.truncate();
                // s.yaw = forward.z.atan2(forward.x);
                // s.pitch = forward.y.asin();
            } else if s.active {
                // Drag in progress
                let dx = mouse_pos.x - s.last_mouse[0];
                let dy = mouse_pos.y - s.last_mouse[1];
                s.last_mouse = [mouse_pos.x, mouse_pos.y];

                s.yaw -= dx * 0.05;
                s.pitch = (s.pitch - dy * 0.05)
                    .clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

                // Build new forward vector
                let cos_pitch = s.pitch.cos();
                let forward = Vec3::new(s.yaw.cos() * cos_pitch, s.pitch.sin(), s.yaw.sin() * cos_pitch);

                // Rotate view matrix: convert to glam, apply yaw/pitch
                let view_glam: Mat4 = Mat4::from(view_matrix);

                // inverse to get camera position & orientation
                let inv_view = view_glam.inverse();
                let cam_pos = Vec3::new(inv_view.w_axis.x, inv_view.w_axis.y, inv_view.w_axis.z);

                let up = Vec3::Y;
                let new_view = Mat4::look_at_lh(cam_pos, cam_pos + forward, up);

                view_out = Some(mint::ColumnMatrix4::from(new_view));
            }
        } else {
            s.active = false;
        }
    });

    if view_out.is_some() {
        return view_out;
    }

    None
}
