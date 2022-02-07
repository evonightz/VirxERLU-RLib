use crate::utils::get_samples_from_path;
use dubins_paths::DubinsPath;
use glam::Vec3A;

#[derive(Clone, Debug)]
pub struct Shot {
    pub time: f32,
    pub distances: [f32; 4],
    pub all_samples: Vec<(f32, f32)>,
    pub samples: [Vec<Vec3A>; 3],
    pub path: DubinsPath,
}

impl Shot {
    const STEP_DISTANCE: f32 = 10.;

    pub fn from(time: f32, path: DubinsPath, distances: [f32; 4]) -> Self {
        // the distance of each segment
        let segment_distances = [path.segment_length(0), path.segment_length(0) + path.segment_length(1), path.length()];

        let all_samples;
        let samples;

        {
            // the samples for each subpath
            let raw_samples = [
                get_samples_from_path(&path, 0., segment_distances[0], Self::STEP_DISTANCE),
                get_samples_from_path(&path, segment_distances[0], segment_distances[1], Self::STEP_DISTANCE),
                get_samples_from_path(&path, segment_distances[1], segment_distances[2], Self::STEP_DISTANCE),
            ];

            all_samples = raw_samples[0]
                .iter()
                .map(|x| (x[0], x[1]))
                .chain(raw_samples[1].iter().map(|x| (x[0], x[1])))
                .chain(raw_samples[2].iter().map(|x| (x[0], x[1])))
                .collect();

            samples = [
                raw_samples[0].iter().map(|v| Vec3A::new(v[0], v[1], 0.)).collect(),
                raw_samples[1].iter().map(|v| Vec3A::new(v[0], v[1], 0.)).collect(),
                raw_samples[2].iter().map(|v| Vec3A::new(v[0], v[1], 0.)).collect(),
            ];
        }

        Self {
            time,
            distances,
            all_samples,
            samples,
            path,
        }
    }

    fn find_min_distance_in_segment_index(&self, segment: usize, target: Vec3A) -> (usize, f32) {
        let mut min_distance = f32::MAX;
        let mut start_index = 0;

        let length = self.samples[segment].len();
        if length == 0 {
            return (start_index, min_distance);
        }

        let mut end_index = length - 1;

        while start_index < end_index {
            let mid_index = (start_index + end_index) / 2;
            min_distance = self.samples[segment][mid_index].distance(target);

            if min_distance < self.samples[segment][mid_index + 1].distance(target) {
                end_index = mid_index;
            } else {
                start_index = mid_index + 1;
            }
        }

        (start_index, min_distance)
    }

    fn find_min_distance_index(&self, target: Vec3A) -> (usize, usize) {
        let mut min_distance = f32::MAX;
        let mut min_distance_index = 0;
        let mut min_distance_index_in_section = 0;

        for segment in 0..3 {
            let (index, distance) = self.find_min_distance_in_segment_index(segment, target);

            if distance < min_distance {
                min_distance = distance;
                min_distance_index = segment;
                min_distance_index_in_section = index;
            }
        }

        (min_distance_index, min_distance_index_in_section)
    }

    pub fn get_distance_along_shot_and_index(&self, target: Vec3A) -> (f32, usize) {
        let (segment, index) = self.find_min_distance_index(target);

        let pre_distance = match segment {
            0 => 0.,
            1 => self.distances[0],
            2 => self.distances[0] + self.distances[1],
            _ => unreachable!(),
        };

        let pre_index = match segment {
            0 => 0,
            1 => self.samples[0].len(),
            2 => self.samples[0].len() + self.samples[1].len(),
            _ => unreachable!(),
        };

        (pre_distance + index as f32 * Self::STEP_DISTANCE, pre_index + index)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub all: bool,
    pub use_absolute_max_values: bool,
    pub min_slice: usize,
    pub max_slice: usize,
}

impl Options {
    pub fn from(min_slice: Option<usize>, max_slice: Option<usize>, use_absolute_max_values: Option<bool>, all: Option<bool>, max_slices: usize) -> Self {
        let min_slice = min_slice.unwrap_or(0);
        let max_slice = max_slice.unwrap_or(max_slices);
        let use_absolute_max_values = use_absolute_max_values.unwrap_or(false);
        let all = all.unwrap_or(false);

        Self {
            all,
            use_absolute_max_values,
            min_slice,
            max_slice,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Target {
    pub car_index: usize,
    pub target_left: Vec3A,
    pub target_right: Vec3A,
    pub options: Options,
    pub shot: Option<Shot>,
    confirmed: bool,
}

impl Target {
    pub const fn new(target_left: Vec3A, target_right: Vec3A, car_index: usize, options: Options) -> Self {
        Self {
            car_index,
            target_left,
            target_right,
            options,
            shot: None,
            confirmed: false,
        }
    }

    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    pub const fn is_confirmed(&self) -> bool {
        self.confirmed
    }
}
