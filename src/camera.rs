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
    use super::*;

    const POSITION: Point3<f32> = cgmath::Point3::<f32>::new(1.0, 2.0, 3.0);

    #[test]
    fn should_set_projection_matrix() {
        let expected_projection_matrix = Matrix4::<f32>::new(
            1.1772641, 0.0, 0.0, 0.0, 0.0, 1.5696855, 0.0, 0.0, 0.0, 0.0, -1.002002, -1.0, 0.0, 0.0, -0.2002002, 0.0,
        );
        let mut camera = Camera::<f32>::default();

        camera.set_projection_matrix(800.0, 600.0, 65.0, 0.1, 100.0);

        assert_eq!(expected_projection_matrix, camera.projection);
    }

    #[test]
    fn should_set_position() {
        let mut camera = Camera::<f32>::default();

        camera.set_position(POSITION);

        assert_eq!(camera.position.x, POSITION[0]);
        assert_eq!(camera.position.y, POSITION[1]);
        assert_eq!(camera.position.z, POSITION[2]);
    }

    #[test]
    fn should_look_at_point() {
        let expected_view_matrix = Matrix4::new(
            0.94868326,
            -0.09534626,
            0.30151135,
            0.0,
            0.0,
            0.9534626,
            0.30151135,
            0.0,
            -0.31622776,
            -0.28603876,
            0.90453404,
            0.0,
            -0.0,
            -0.9534627,
            -3.6181362,
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
