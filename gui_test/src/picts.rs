use application::image::*;

pub struct Picts {
    pub end_point: Image<bool>,
    pub tangent_point: Image<bool>,
    pub cross_point: Image<bool>,
    pub grid_point: Image<bool>,
    pub center_point: Image<bool>,
}

impl Picts {
    pub fn new() -> Self {
        let size = 17;
        let grid_point_size = 27;
        let mut end_point = Image::new((size, size));
        let mut tangent_point = Image::new((size, size));
        let mut cross_point = Image::new((size, size));
        let mut grid_point = Image::new((grid_point_size, grid_point_size));
        let mut center_point = Image::new((size, size));

        end_point
            .as_view_mut()
            .fill_with_coord(|p, (x, y)| *p = x == 0 || x == size - 1 || y == 0 || y == size - 1);

        cross_point
            .as_view_mut()
            .fill_with_coord(|p, (x, y)| *p = x == y || x + y == size - 1);

        tangent_point.as_view_mut().fill_with_coord(|p, (x, y)| {
            *p = if y == 0 {
                true
            } else {
                let size = size as i32;
                let x = x as i32 * 2 - (size - 1);
                let y = y as i32 * 2 - (size - 1);
                let r2 = x * x + y * y;
                r2 <= (size - 3) * (size - 3) + 4 && r2 > (size - 5) * (size - 5) + 4
            };
        });

        // grid_point
        //     .as_view_mut()
        //     .fill_with_coord(|p, (x, y)| *p = x == grid_point_size / 2 || y == grid_point_size / 2);

        grid_point.as_view_mut().fill_with_coord(|p, (x, y)| {
            let size = grid_point_size as i32;
            let x = (x as i32 * 2 - (size - 1)).abs();
            let y = (y as i32 * 2 - (size - 1)).abs();
            *p = (x == 2 || y == 2) && x >= 2 && y >= 2;
        });

        center_point.as_view_mut().fill_with_coord(|p, (x, y)| {
            let size = size as i32;
            let x = x as i32 * 2 - (size - 1);
            let y = y as i32 * 2 - (size - 1);
            let r2 = x * x + y * y;
            *p = r2 <= (size - 1) * (size - 1) + 16 && r2 > (size - 3) * (size - 3) + 16;
        });

        Self {
            end_point,
            tangent_point,
            cross_point,
            grid_point,
            center_point,
        }
    }
}
