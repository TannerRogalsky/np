use crate::Float;

#[derive(Debug, Clone, Copy, Default)]
pub enum AABB {
    #[default]
    Unset,
    Set(Rectangle),
}

impl AABB {
    pub fn extend(self, x: Float, y: Float) -> Self {
        match self {
            AABB::Unset => AABB::Set(Rectangle {
                x1: x,
                y1: y,
                x2: x,
                y2: y,
            }),
            AABB::Set(rectangle) => AABB::Set(rectangle.extend(x, y)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x1: Float,
    pub y1: Float,
    pub x2: Float,
    pub y2: Float,
}

impl Rectangle {
    pub fn extend(self, x: Float, y: Float) -> Self {
        let Self { x1, y1, x2, y2 } = self;
        Self {
            x1: x1.min(x),
            y1: y1.min(y),
            x2: x2.max(x),
            y2: y2.max(y),
        }
    }

    pub fn width(&self) -> Float {
        (self.x1 - self.x2).abs()
    }

    pub fn height(&self) -> Float {
        (self.y1 - self.y2).abs()
    }

    pub fn points(&self) -> [Float; 8] {
        [
            self.x1, self.y1, self.x2, self.y1, self.x2, self.y2, self.x1, self.y2,
        ]
    }
}

pub struct Triangle {
    pub cx: f32,
    pub cy: f32,
    pub angle: f32,
    pub radius: f32,
}

impl Triangle {
    pub fn node(&self) -> svg::node::element::Polygon {
        let mut points = vec![0.; 6];
        let delta = std::f32::consts::FRAC_PI_3 * 2.;
        for i in 0..3 {
            let phi = (i as f32) * delta + std::f32::consts::FRAC_PI_2 - self.angle;
            let (x, y) = phi.sin_cos();

            points[i * 2 + 0] = self.cx + x * self.radius;
            points[i * 2 + 1] = self.cy + y * self.radius;
        }
        svg::node::element::Polygon::new().set("points", points)
    }
}
