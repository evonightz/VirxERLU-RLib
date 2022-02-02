mod car;
mod constants;
mod ground;
mod pytypes;
mod shot;
mod utils;

use car::{turn_radius, Car};
use constants::*;
use glam::Vec3A;
use ground::*;
use pyo3::{exceptions, prelude::*, PyErr};
use pytypes::*;
use rl_ball_sym::simulation::{
    ball::{Ball, BallPrediction},
    game::Game,
};
use shot::{Options, Shot, Target};
use utils::*;

static mut GAME_TIME: f32 = 0.;

static mut GAME: Option<Game> = None;
const NO_GAME_ERR: &str = "GAME is unset. Call a function like load_soccar first.";

static mut CARS: Option<Vec<Car>> = None;
const NO_CARS_ERR: &str = "CARS is unset. Call a function like load_soccar first.";
const NO_CAR_ERR: &str = "No car at the provided index.";

static mut BALL_STRUCT: Option<BallPrediction> = None;
const NO_BALL_STRUCT_ERR: &str = "BALL_STRUCT is unset. Call the function tick and pass in game information first.";

static mut TARGETS: Vec<Option<Target>> = Vec::new();
const NO_TARGET_ERR: &str = "Target no longer exists.";
const NO_SHOT_ERR: &str = "Specified target has no found shot.";

/// VirxERLU-RLib is written in Rust with Python bindings to make analyzing the ball prediction struct much faster.
#[pymodule]
fn virx_erlu_rlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_soccar, m)?)?;
    m.add_function(wrap_pyfunction!(load_dropshot, m)?)?;
    m.add_function(wrap_pyfunction!(load_hoops, m)?)?;
    m.add_function(wrap_pyfunction!(load_soccar_throwback, m)?)?;
    m.add_function(wrap_pyfunction!(tick, m)?)?;
    m.add_function(wrap_pyfunction!(get_slice, m)?)?;
    m.add_function(wrap_pyfunction!(new_target, m)?)?;
    m.add_function(wrap_pyfunction!(confirm_target, m)?)?;
    m.add_function(wrap_pyfunction!(remove_target, m)?)?;
    m.add_function(wrap_pyfunction!(print_targets, m)?)?;
    m.add_function(wrap_pyfunction!(get_shot_with_target, m)?)?;
    m.add_function(wrap_pyfunction!(get_data_for_shot_with_target, m)?)?;
    Ok(())
}

#[pyfunction]
fn load_soccar() {
    unsafe {
        GAME = Some(rl_ball_sym::load_soccar());
        CARS = Some(vec![Car::default(); 64]);
        BALL_STRUCT = Some(BallPrediction::default());
    }
}

#[pyfunction]
fn load_dropshot() {
    unsafe {
        GAME = Some(rl_ball_sym::load_dropshot());
        CARS = Some(vec![Car::default(); 64]);
        BALL_STRUCT = Some(BallPrediction::default());
    }
}

#[pyfunction]
fn load_hoops() {
    unsafe {
        GAME = Some(rl_ball_sym::load_hoops());
        CARS = Some(vec![Car::default(); 64]);
        BALL_STRUCT = Some(BallPrediction::default());
    }
}

#[pyfunction]
fn load_soccar_throwback() {
    unsafe {
        GAME = Some(rl_ball_sym::load_soccar_throwback());
        CARS = Some(vec![Car::default(); 64]);
        BALL_STRUCT = Some(BallPrediction::default());
    }
}

#[pyfunction]
fn tick(py: Python, packet: PyObject, prediction_time: Option<f32>) -> PyResult<()> {
    let game: &mut Game;
    let cars: &mut Vec<Car>;
    let targets: &mut Vec<Option<Target>>;

    // simulate max jump height
    // simulate max double jump height
    // add option for max path time

    unsafe {
        targets = &mut TARGETS;
        game = GAME.as_mut().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_GAME_ERR))?;
        cars = CARS.as_mut().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_CARS_ERR))?;
    }

    targets.retain(|target| match target {
        Some(t) => t.is_confirmed(),
        None => true,
    });

    /*
    Example GameTickPacket<
        game_cars: [
            PlayerInfo<
                physics: Physics<
                    location: Vector3<x: 0.0, y: -4608.0, z: 17.010000228881836>,
                    rotation: Rotator<pitch: -0.009587380103766918, yaw: 1.5707963705062866, roll: 0.0>,
                    velocity: Vector3<x: 0.0, y: 0.0, z: 0.210999995470047>,
                    angular_velocity: Vector3<x: -0.0006099999882280827, y: 0.0, z: 0.0>
                >,
                score_info: ScoreInfo<score: 0, goals: 0, own_goals: 0, assists: 0, saves: 0, shots: 0, demolitions: 0>,
                is_demolished: False,
                has_wheel_contact: True,
                is_super_sonic: False,
                is_bot: True,
                jumped: False,
                double_jumped: False,
                name: 'DownToEarth',
                team: 0,
                boost: 34,
                hitbox: BoxShape<length: 118.00737762451172, width: 84.19940948486328, height: 36.15907287597656>,
                hitbox_offset: Vector3<x: 13.875659942626953, y: 0.0, z: 20.754987716674805>,
                spawn_id: 1793714700
            >
        ],
        num_cars: 1,
        game_boosts: <rlbot.utils.structures.game_data_struct.BoostPadState_Array_50 object at 0x000002C910DE8EC8>,
        num_boost: 34,
        game_ball: BallInfo<
            physics: Physics<location: Vector3<x: 0.0, y: 0.0, z: 92.73999786376953>, rotation: Rotator<pitch: 0.0, yaw: 0.0, roll: 0.0>, velocity: Vector3<x: 0.0, y: 0.0, z: 0.0>, angular_velocity: Vector3<x: 0.0, y: 0.0, z: 0.0>>,
            latest_touch: Touch<player_name: '', time_seconds: 0.0, hit_location: Vector3<x: 0.0, y: 0.0, z: 0.0>, hit_normal: Vector3<x: 0.0, y: 0.0, z: 0.0>, team: 0, player_index: 0>,
            drop_shot_info: DropShotInfo<absorbed_force: 0.0, damage_index: 0, force_accum_recent: 0.0>,
            collision_shape: CollisionShape<type: 1, box: BoxShape<length: 0.0, width: 0.0, height: 0.0>,
            sphere: SphereShape<diameter: 182.49998474121094>, cylinder: CylinderShape<diameter: 0.0, height: 0.0>>
        >,
        game_info: GameInfo<seconds_elapsed: 718.4749755859375, game_time_remaining: -707.4849243164062, is_overtime: False, is_unlimited_time: True, is_round_active: False, is_kickoff_pause: False, is_match_ended: False, world_gravity_z: -650.0, game_speed: 0.0, frame_num: 86217>,
        dropshot_tiles: <rlbot.utils.structures.game_data_struct.TileInfo_Array_200 object at 0x000002C910DE8EC8>,
        num_tiles: 0,
        teams: <rlbot.utils.structures.game_data_struct.TeamInfo_Array_2 object at 0x000002C910DE8EC8>,
        num_teams: 2
    >
    */
    let packet = packet.as_ref(py);

    let py_game_info = packet.getattr("game_info")?;

    let time = py_game_info.getattr("seconds_elapsed")?.extract()?;

    unsafe {
        GAME_TIME = time;
    }

    game.gravity.z = py_game_info.getattr("world_gravity_z")?.extract()?;

    let py_ball = packet.getattr("game_ball")?;
    let py_ball_physics = py_ball.getattr("physics")?;

    game.ball.update(
        time,
        get_vec3_named(py_ball_physics.getattr("location")?)?,
        get_vec3_named(py_ball_physics.getattr("velocity")?)?,
        get_vec3_named(py_ball_physics.getattr("angular_velocity")?)?,
    );

    let py_ball_shape = py_ball.getattr("collision_shape")?;

    game.ball.radius = py_ball_shape.getattr("sphere")?.getattr("diameter")?.extract::<f32>()? / 2.;
    game.ball.collision_radius = game.ball.radius + 1.9;
    game.ball.calculate_moi();

    let prediction_time = prediction_time.unwrap_or(6.);
    let ball_struct = Ball::get_ball_prediction_struct_for_time(game, &prediction_time);

    unsafe {
        BALL_STRUCT = Some(ball_struct);
    }

    let num_cars = packet.getattr("num_cars")?.extract::<usize>()?;
    let py_game_cars = packet.getattr("game_cars")?;

    for (i, car) in cars.iter_mut().enumerate().take(num_cars) {
        car.update(py_game_cars.get_item(i)?)?;
    }

    Ok(())
}

#[pyfunction]
fn get_slice(slice_time: f32) -> PyResult<BallSlice> {
    let game_time: &f32;
    let ball_struct: &BallPrediction;

    unsafe {
        game_time = &GAME_TIME;
        ball_struct = BALL_STRUCT.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_BALL_STRUCT_ERR))?;
    }

    let slice_num = ((slice_time - game_time) * 120.).round() as usize;
    let ball = ball_struct.slices[slice_num.clamp(1, ball_struct.num_slices) - 1];

    Ok(BallSlice::from(&ball))
}

#[pyfunction]
fn new_target(
    left_target: Vec<f32>,
    right_target: Vec<f32>,
    car_index: usize,
    min_slice: Option<usize>,
    max_slice: Option<usize>,
    use_absolute_max_values: Option<bool>,
    all: Option<bool>,
) -> PyResult<usize> {
    let num_slices: usize;

    {
        let ball_prediction: &BallPrediction;

        unsafe {
            ball_prediction = BALL_STRUCT.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_BALL_STRUCT_ERR))?;
        }

        num_slices = ball_prediction.num_slices;
    }

    let target_left = get_vec3_from_vec(left_target, "target_left")?;
    let target_right = get_vec3_from_vec(right_target, "target_right")?;
    let options = Options::from(min_slice, max_slice, use_absolute_max_values, all, num_slices);

    let target = Some(Target::new(target_left, target_right, car_index, options));
    let target_index;

    unsafe {
        target_index = match TARGETS.iter().position(|x| x.is_none()) {
            Some(i) => {
                TARGETS[i] = target;
                i
            }
            None => {
                TARGETS.push(target);
                TARGETS.len() - 1
            }
        };
    }

    Ok(target_index)
}

#[pyfunction]
fn confirm_target(target_index: usize) -> PyResult<()> {
    let target;

    unsafe {
        target = match TARGETS[target_index].as_mut() {
            Some(t) => t,
            None => {
                return Err(PyErr::new::<exceptions::PyIndexError, _>(NO_TARGET_ERR));
            }
        }
    }

    target.confirm();

    Ok(())
}

#[pyfunction]
fn remove_target(target_index: usize) -> PyResult<()> {
    unsafe {
        match TARGETS.get(target_index) {
            Some(t) => {
                if t.is_none() {
                    return Err(PyErr::new::<exceptions::PyIndexError, _>(NO_TARGET_ERR));
                }

                TARGETS[target_index] = None;
            }
            None => {
                return Err(PyErr::new::<exceptions::PyIndexError, _>(NO_TARGET_ERR));
            }
        }
    }

    Ok(())
}

#[pyfunction]
fn print_targets() {
    unsafe {
        dbg!(&TARGETS);
    }
}

#[pyfunction]
fn get_shot_with_target(target_index: usize, temporary: Option<bool>) -> PyResult<BasicShotInfo> {
    let game_time: &f32;
    let _gravity: &Vec3A;
    let radius: &f32;
    let car: &Car;
    let ball_prediction: &BallPrediction;
    let target: &mut Target;

    {
        let game: &Game;
        let cars: &mut Vec<Car>;

        unsafe {
            game_time = &GAME_TIME;
            game = GAME.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_GAME_ERR))?;
            ball_prediction = BALL_STRUCT.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_BALL_STRUCT_ERR))?;
            target = TARGETS[target_index].as_mut().ok_or_else(|| PyErr::new::<exceptions::PyIndexError, _>(NO_TARGET_ERR))?;
            cars = CARS.as_mut().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_CARS_ERR))?;
        }

        _gravity = &game.gravity;
        radius = &game.ball.radius;
        car = cars.get_mut(target.car_index).ok_or_else(|| PyErr::new::<exceptions::PyIndexError, _>(NO_CAR_ERR))?;
    }

    let dist_from_side = radius + car.hitbox.height;

    let mut found_shot = None;
    let mut found_time = None;

    if ball_prediction.num_slices == 0 || car.demolished || car.airborne {
        return Ok(BasicShotInfo::not_found());
    }

    let max_speed = if target.options.use_absolute_max_values { MAX_SPEED } else { car.max_speed };

    let max_turn_radius = if target.options.use_absolute_max_values { turn_radius(MAX_SPEED) } else { car.ctrms };

    let analyze_options = AnalyzeOptions {
        max_speed,
        max_turn_radius,
        get_target: false,
        validate: true,
    };

    let temporary = temporary.unwrap_or(false);

    for ball in &ball_prediction.slices[target.options.min_slice..target.options.max_slice] {
        if ball.location.y.abs() > 5120. + ball.collision_radius {
            break;
        }

        if ball.location.z > dist_from_side {
            continue;
        }

        let car_to_ball = ball.location - car.location;

        let post_info = correct_for_posts(ball.location, ball.collision_radius, target.target_left, target.target_right);

        if !post_info.fits {
            continue;
        }

        let shot_vector = get_shot_vector_2d(
            flatten(car_to_ball).normalize_or_zero(),
            flatten(ball.location),
            flatten(post_info.target_left),
            flatten(post_info.target_right),
        );
        let max_time_remaining = ball.time - game_time;
        let result = match analyze_target(ball, car, shot_vector, max_time_remaining, analyze_options) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let distance_remaining = result.distances.iter().sum();
        let is_forwards = true;

        // will be used to calculate if there's enough time left to jump after accelerating
        let _time_remaining = match can_reach_target(car, max_speed, max_time_remaining, distance_remaining, is_forwards) {
            Ok(t_r) => t_r,
            Err(_) => continue,
        };

        if found_shot.is_none() {
            found_time = Some(ball.time);

            if !temporary {
                found_shot = Some(Shot::from(ball.time, result.path, result.distances));
            }

            if !target.options.all {
                break;
            }
        }
    }

    target.shot = found_shot;

    Ok(match found_time {
        Some(time) => BasicShotInfo::found(time),
        None => BasicShotInfo::not_found(),
    })
}

#[pyfunction]
fn get_data_for_shot_with_target(target_index: usize) -> PyResult<AdvancedShotInfo> {
    let game_time: &f32;
    let _gravity: &Vec3A;
    let car: &Car;
    let ball: &Ball;
    let target: &Target;
    let shot: &Shot;

    {
        let game: &Game;
        let cars: &mut Vec<Car>;
        let ball_struct: &BallPrediction;

        unsafe {
            game_time = &GAME_TIME;
            game = GAME.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_GAME_ERR))?;
            target = TARGETS[target_index].as_ref().ok_or_else(|| PyErr::new::<exceptions::PyIndexError, _>(NO_TARGET_ERR))?;
            cars = CARS.as_mut().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_CARS_ERR))?;
            ball_struct = BALL_STRUCT.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyNameError, _>(NO_BALL_STRUCT_ERR))?;
        }

        _gravity = &game.gravity;
        car = cars.get_mut(target.car_index).ok_or_else(|| PyErr::new::<exceptions::PyIndexError, _>(NO_CAR_ERR))?;
        shot = target.shot.as_ref().ok_or_else(|| PyErr::new::<exceptions::PyLookupError, _>(NO_SHOT_ERR))?;

        let slice_num = ((shot.time - game_time) * 120.).round() as usize;
        ball = &ball_struct.slices[slice_num.clamp(1, ball_struct.num_slices) - 1];
    }

    let car_to_ball = ball.location - car.location;
    let post_info = correct_for_posts(ball.location, ball.collision_radius, target.target_left, target.target_right);
    let shot_vector = get_shot_vector_2d(
        flatten(car_to_ball).normalize_or_zero(),
        flatten(ball.location),
        flatten(post_info.target_left),
        flatten(post_info.target_right),
    );

    Ok(AdvancedShotInfo::get(car, shot, shot_vector))
}
