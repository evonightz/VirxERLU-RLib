use crate::{
    car::Car,
    ground::{angle_2d, get_turn_exit_tanget, shortest_path_in_validate, TargetInfo},
    pytypes::ShotType,
    utils::flatten,
};
use dubins_paths::{DubinsPath, NoPathError, PathType, PosRot, Result as DubinsResult};
use glam::Vec3A;
use rl_ball_sym::simulation::ball::Ball;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, Default)]
pub struct Analyzer {
    max_speed: Option<f32>,
    max_turn_radius: Option<f32>,
    gravity: f32,
    may_ground_shot: bool,
    may_jump_shot: bool,
    may_double_jump_shot: bool,
}

impl Analyzer {
    pub const fn new(max_speed: Option<f32>, max_turn_radius: Option<f32>, gravity: f32, may_ground_shot: bool, may_jump_shot: bool, may_double_jump_shot: bool) -> Self {
        Self {
            max_speed,
            max_turn_radius,
            gravity,
            may_ground_shot,
            may_jump_shot,
            may_double_jump_shot,
        }
    }

    fn get_max_speed(&self, car: &Car, slice_num: usize) -> f32 {
        self.max_speed.unwrap_or_else(|| car.max_speed[slice_num])
    }

    fn get_max_turn_radius(&self, car: &Car, slice_num: usize) -> f32 {
        self.max_turn_radius.unwrap_or_else(|| car.ctrms[slice_num])
    }

    /// get the type of shot that will be required to hit the ball
    /// also check if that type of shot has been enabled
    fn get_shot_type(&self, car: &Car, target: Vec3A) -> DubinsResult<usize> {
        if target.z < car.hitbox.height / 2. + 17. {
            match self.may_ground_shot {
                true => Ok(ShotType::GROUND),
                false => Err(NoPathError),
            }
        } else if target.z < car.max_jump_height {
            match self.may_jump_shot {
                true => Ok(ShotType::JUMP),
                false => Err(NoPathError),
            }
        } else if target.z < car.max_double_jump_height {
            match self.may_double_jump_shot {
                true => Ok(ShotType::DOUBLE_JUMP),
                false => Err(NoPathError),
            }
        } else {
            Err(NoPathError)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn get_jump_info(
        &self,
        car: &Car,
        ball_location: Vec3A,
        target: Vec3A,
        shot_vector: Vec3A,
        max_speed: f32,
        time_remaining: f32,
        shot_type: usize,
    ) -> DubinsResult<(Option<f32>, f32)> {
        Ok(match shot_type {
            ShotType::GROUND => {
                let distance = 320.;
                (
                    None,
                    if (0_f32..distance).contains(&car.forward.dot(ball_location))
                        && car.right.dot(ball_location) < car.hitbox.width / 2.
                        && angle_2d(car.forward, shot_vector) < 0.02
                    {
                        0. // for pre-aligned ground shots
                    } else {
                        distance // for non-aligned ground shots
                    },
                )
            }
            ShotType::JUMP => {
                let time = car.jump_time_to_height(self.gravity, target.z - car.hitbox.height / 2.);

                (Some(time), time * max_speed + 128.)
            }
            ShotType::DOUBLE_JUMP => {
                // if we need to do a double jump but we don't even have time for a normal jump
                if time_remaining > car.max_jump_time {
                    return Err(NoPathError);
                }

                let time = car.double_jump_time_to_height(self.gravity, target.z - car.hitbox.height / 2.);

                (Some(time), time * max_speed + 128.)
            }
            _ => unreachable!(),
        })
    }

    pub fn no_target(&self, ball: &Ball, car: &Car, time_remaining: f32, slice_num: usize) -> DubinsResult<TargetInfo> {
        let car_front_length = (car.hitbox_offset.x + car.hitbox.length) / 2.;

        // we only have ground-based shots right now
        // check if we've landed on the ground at this point in time
        if car.landing_time >= time_remaining {
            return Err(NoPathError);
        }

        let max_speed = self.get_max_speed(car, slice_num);

        let time_remaining = time_remaining - car.landing_time;
        let car_location = flatten(car.landing_location);
        let max_distance = time_remaining * max_speed + car_front_length + ball.radius;

        // check if a simplified path is longer than the longest distance we can possibly travel
        if car_location.distance(flatten(ball.location)) > max_distance {
            return Err(NoPathError);
        }

        let shot_type = self.get_shot_type(car, ball.location)?;
        let (jump_time, end_distance) = if shot_type != ShotType::GROUND {
            self.get_jump_info(
                car,
                ball.location,
                ball.location,
                (ball.location - car.location).normalize_or_zero(),
                max_speed,
                time_remaining,
                shot_type,
            )?
        } else {
            (None, 0.)
        };

        let offset_distance = end_distance - car_front_length - ball.radius;

        if let Some(jump_time) = jump_time {
            // if we have enough time for just the jump
            if jump_time > time_remaining {
                return Err(NoPathError);
            }
        }

        let rho = self.get_max_turn_radius(car, slice_num);
        let is_forwards = true;
        let target_is_forwards = car.forward.dot(ball.location) >= 0.;
        let should_turn_left = car.right.dot(ball.location) < 0.;
        let center_of_turn = flatten(if car.right.dot(ball.location) < 0. { -car.landing_right } else { car.landing_right } * rho);

        let turn_target = car_location + get_turn_exit_tanget(car, flatten(ball.location), center_of_turn, rho, target_is_forwards);

        // compute the distance of each path, validating that it is within our current maximum travel distance (returning an error if neither are)

        let turn_final_distance = turn_target.distance(ball.location) - ball.radius - car_front_length;

        if turn_final_distance < offset_distance || turn_final_distance + turn_target.distance(car_location) > max_distance {
            return Err(NoPathError);
        }

        // find the angle between the car location and each turn exit point relative to the exit turn point centers
        let turn_angle = angle_2d(car_location - center_of_turn, turn_target - center_of_turn);
        let turn_arc_distance = turn_angle * rho;

        if turn_final_distance + turn_arc_distance > max_distance {
            return Err(NoPathError);
        }

        let shot_vector = (flatten(ball.location) - turn_target).normalize_or_zero();

        // check if the exit point is in the field, and make sure a simplified version of the path isn't longer than the longest distance we can travel
        if !car.field.is_point_in(turn_target) {
            return Err(NoPathError);
        }

        // construct a path so we can easily follow our defined turn arc
        let path = DubinsPath {
            qi: PosRot::new(car.landing_location, car.landing_yaw),
            rho,
            type_: if should_turn_left { PathType::LSR } else { PathType::RSL },
            param: [turn_arc_distance / rho, 0., 0.],
        };

        let distances = [turn_arc_distance, 0., 0., turn_final_distance];

        Ok(TargetInfo::from(distances, shot_type, path, jump_time, is_forwards, shot_vector))
    }

    pub fn target(&self, ball: &Ball, car: &Car, shot_vector: Vec3A, time_remaining: f32, slice_num: usize) -> DubinsResult<TargetInfo> {
        let offset_target = ball.location - (shot_vector * ball.radius);
        let car_front_length = (car.hitbox_offset.x + car.hitbox.length) / 2.;

        // we only have ground-based shots right now
        // check if we've landed on the ground at this point in time
        if car.landing_time >= time_remaining {
            return Err(NoPathError);
        }

        let max_speed = self.get_max_speed(car, slice_num);

        let time_remaining = time_remaining - car.landing_time;
        let car_location = car.landing_location;
        let max_distance = time_remaining * max_speed + car_front_length;

        // check if a simplified path is longer than the longest distance we can possibly travel
        if flatten(car_location).distance(flatten(offset_target)) > max_distance {
            return Err(NoPathError);
        }

        let shot_type = self.get_shot_type(car, ball.location)?;
        let (jump_time, end_distance) = self.get_jump_info(car, ball.location, offset_target, shot_vector, max_speed, time_remaining, shot_type)?;

        if let Some(jump_time) = jump_time {
            // if we have enough time for just the jump
            if jump_time > time_remaining {
                return Err(NoPathError);
            }
        }

        let offset_distance = end_distance - car_front_length;
        let exit_turn_target = flatten(offset_target) - (flatten(shot_vector).normalize_or_zero() * end_distance);

        // check if the exit point is in the field, and make sure a simplified version of the path isn't longer than the longest distance we can travel
        if !car.field.is_point_in(flatten(exit_turn_target)) || flatten(car_location).distance(exit_turn_target) + end_distance > max_distance {
            return Err(NoPathError);
        }

        // calculate and return the dubin's path

        let target_angle = shot_vector.y.atan2(shot_vector.x);
        let mut starting_yaw = car.landing_yaw;

        // it's easier for me to think about what I want the criteria to be for going backwards, so I did that then just took the opposite of it for is_forwards
        let is_backwards = time_remaining < 4. && angle_2d(shot_vector, Vec3A::new(car.landing_yaw.cos(), car.landing_yaw.sin(), 0.)) > PI * (2. / 3.);
        let is_forwards = !is_backwards;

        if !is_forwards {
            starting_yaw += PI;
        }

        let q0 = PosRot::new(flatten(car_location), starting_yaw);
        let q1 = PosRot::new(flatten(exit_turn_target), target_angle);

        let path = shortest_path_in_validate(q0, q1, self.get_max_turn_radius(car, slice_num), &car.field, max_distance)?;

        let distances = [path.segment_length(0), path.segment_length(1), path.segment_length(2), offset_distance];

        Ok(TargetInfo::from(distances, shot_type, path, jump_time, is_forwards, shot_vector))
    }
}
