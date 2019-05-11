use cgmath::{BaseFloat, Deg, EuclideanSpace, Matrix4, PerspectiveFov, Point3, SquareMatrix, Vector3};

pub struct Camera<S>
where
    S: BaseFloat,
{
    position: Point3<S>,
    view: Matrix4<S>,
    projection: Matrix4<S>,
}

impl<S> Default for Camera<S>
where
    S: BaseFloat,
{
    fn default() -> Camera<S> {
        let identity = Matrix4::<S>::identity();

        Camera {
            position: Point3::<S>::new(S::zero(), S::zero(), S::zero()),

            view: identity,
            projection: identity,
        }
    }
}

impl<S> Camera<S>
where
    S: BaseFloat,
{
    pub fn get_position(&self) -> Point3<S> {
        self.position
    }

    pub fn set_position(&mut self, position: Point3<S>) {
        self.position = position;
    }

    pub fn look_at(&mut self, target: Point3<S>, up: Vector3<S>) {
        self.view = cgmath::Matrix4::look_at(self.position, target, up);
    }

    pub fn get_mvp_matrix(&self) -> Matrix4<S> {
        self.projection * self.view * Matrix4::from_translation(self.position.to_vec())
        // self.projection * self.view * self.model
    }

    pub fn get_mvp_matrix_array(&self) -> [[S; 4]; 4] {
        cgmath::conv::array4x4(self.get_mvp_matrix())
    }

    /// Sets the Camera's projection matrix from the provided params.
    pub fn set_projection_matrix(&mut self, width: S, height: S, fov_deg: S, z_near: S, z_far: S) {
        let perspective_fov = PerspectiveFov {
            fovy: Deg(fov_deg).into(),
            aspect: width / height,
            near: z_near,
            far: z_far,
        };

        self.projection = perspective_fov.into();
    }
}

#[cfg(test)]
pub mod test {
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    use super::*;

    const POSITION: Point3<f32> = cgmath::Point3::<f32>::new(1.0, 2.0, 3.0);

    #[test]
    fn should_set_projection_matrix() {
        let expected_projection_matrix = Matrix4::<f32>::new(
            1.177_264_1,
            0.0,
            0.0,
            0.0,
            0.0,
            1.569_685_5,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.002_002,
            -1.0,
            0.0,
            0.0,
            -0.200_200_2,
            0.0,
        );
        let mut camera = Camera::<f32>::default();

        camera.set_projection_matrix(800.0, 600.0, 65.0, 0.1, 100.0);

        assert_eq!(expected_projection_matrix, camera.projection);
    }

    #[test]
    fn should_set_position() {
        let mut camera = Camera::<f32>::default();

        camera.set_position(POSITION);

        assert_delta!(camera.position.x, POSITION[0], ::std::f32::EPSILON);
        assert_delta!(camera.position.y, POSITION[1], ::std::f32::EPSILON);
        assert_delta!(camera.position.z, POSITION[2], ::std::f32::EPSILON);
    }

    #[test]
    fn should_look_at_point() {
        let expected_view_matrix = Matrix4::new(
            0.948_683_26,
            -0.095_346_26,
            0.301_511_35,
            0.0,
            0.0,
            0.953_462_6,
            0.301_511_35,
            0.0,
            -0.316_227_76,
            -0.286_038_76,
            0.904_534_04,
            0.0,
            -0.0,
            -0.953_462_7,
            -3.618_136_2,
            1.0,
        );

        let target = Point3::new(0.0, 1.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);

        let mut camera = Camera::<f32>::default();
        camera.set_position(POSITION);
        camera.look_at(target, up);

        assert_eq!(expected_view_matrix, camera.view);
    }
}
