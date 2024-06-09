use crate::villager::Direction;
use bevy::math::Vec2;
use extend::ext;
use grid_2d::Coord;

#[ext]
pub impl Coord {
    /// Converts a `Coord` to a `Vec2`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::ext::CoordExt;
    /// use grid_2d::Coord;
    ///
    /// let coord = Coord { x: 3, y: 4 };
    /// let vec = coord.to_vec2();
    /// assert_eq!(vec, Vec2::new(3.0, 4.0));
    /// ```
    fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

#[ext]
pub impl Vec2 {
    /// Converts a `Vec2` to world space (16x16 units).
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::ext::Vec2Ext;
    ///
    /// let vec = Vec2::new(2.0, 3.0);
    /// let world_space_vec = vec.to_world_space();
    /// assert_eq!(world_space_vec, Vec2::new(32.0, 48.0));
    /// ```
    fn to_world_space(&self) -> Vec2 {
        *self * 16.0
    }

    /// Returns a normalized vector pointing from `self` towards `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::ext::Vec2Ext;
    ///
    /// let vec1 = Vec2::new(1.0, 1.0);
    /// let vec2 = Vec2::new(4.0, 5.0);
    /// let direction = vec1.towards(&vec2);
    /// assert!((direction - Vec2::new(0.6, 0.8)).length() < 1e-5);
    /// ```
    fn towards(&self, other: &Vec2) -> Vec2 {
        (*other - *self).normalize()
    }

    /// Converts a `Vec2` to a `Direction`, rounding the vector components.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::ext::Vec2Ext;
    /// use bevy_game::villager::Direction;
    ///
    /// let vec = Vec2::new(0.1, 0.9);
    /// assert_eq!(vec.to_direction(), Some(Direction::Up));
    ///
    /// let invalid_vec = Vec2::new(1.1, 1.1);
    /// assert_eq!(invalid_vec.to_direction(), None);
    /// ```
    fn to_direction(&self) -> Option<Direction> {
        match self.round() {
            v if v == Vec2::new(0.0, 1.0) => Some(Direction::Up),
            v if v == Vec2::new(0.0, -1.0) => Some(Direction::Down),
            v if v == Vec2::new(-1.0, 0.0) => Some(Direction::Left),
            v if v == Vec2::new(1.0, 0.0) => Some(Direction::Right),
            _ => None,
        }
    }

    /// Returns the `Direction` from `self` towards `other`, if the direction is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::ext::Vec2Ext;
    /// use bevy_game::villager::Direction;
    ///
    /// let vec1 = Vec2::new(1.0, 1.0);
    /// let vec2 = Vec2::new(4.0, 1.0);
    /// let direction = vec1.to_direction_towards(&vec2);
    /// assert_eq!(direction, Some(Direction::Right));
    ///
    /// let vec1 = Vec2::new(0.0, 0.0);
    /// let vec2 = Vec2::new(1.0, 0.0);
    /// let direction = vec1.to_direction_towards(&vec2);
    /// assert_eq!(direction, Some(Direction::Right));
    /// ```
    fn to_direction_towards(&self, other: &Vec2) -> Option<Direction> {
        self.towards(other).normalize().to_direction()
    }
}
