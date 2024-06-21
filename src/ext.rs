use crate::worldgen::{TILEMAP_SIZE, TILEMAP_TILE_SIZE, TILEMAP_TYPE};
use bevy::math::Vec2;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::SquareDirection;
use bevy_ecs_tilemap::prelude::TilePos;
use extend::ext;
use grid_2d::Coord;

#[ext]
pub impl TilePos {
    fn to_coord(&self) -> Coord {
        Coord {
            x: self.x as i32,
            y: self.y as i32,
        }
    }

    fn to_world_space(&self) -> Vec2 {
        self.center_in_world(&TILEMAP_TILE_SIZE.into(), &TILEMAP_TYPE)
    }
}

#[ext]
pub impl Vec2 {
    /// Converts a `Vec2` to `TilePos`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_ecs_tilemap::tiles::TilePos;
    /// use bevy_game::ext::Vec2Ext;
    ///
    /// let vec = Vec2::new(2.7, 3.9);
    /// let tilepos = vec.to_tilepos();
    /// assert_eq!(tilepos, TilePos { x: 0, y: 0 });
    /// ```
    fn to_tilepos(&self) -> TilePos {
        TilePos::from_world_pos(
            self,
            &TILEMAP_SIZE,
            &TILEMAP_TILE_SIZE.into(),
            &TILEMAP_TYPE,
        )
        .unwrap()
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
    /// assert_eq!(vec.to_direction(), Some(SquareDirection::North));
    ///
    /// let invalid_vec = Vec2::new(1.1, 1.1);
    /// assert_eq!(invalid_vec.to_direction(), None);
    /// ```
    fn to_direction(&self) -> Option<SquareDirection> {
        match self.round() {
            v if v == Vec2::new(0.0, 1.0) => Some(SquareDirection::North),
            v if v == Vec2::new(0.0, -1.0) => Some(SquareDirection::South),
            v if v == Vec2::new(-1.0, 0.0) => Some(SquareDirection::West),
            v if v == Vec2::new(1.0, 0.0) => Some(SquareDirection::East),
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
    /// assert_eq!(direction, Some(SquareDirection::East));
    ///
    /// let vec1 = Vec2::new(0.0, 0.0);
    /// let vec2 = Vec2::new(1.0, 0.0);
    /// let direction = vec1.to_direction_towards(&vec2);
    /// assert_eq!(direction, Some(SquareDirection::East));
    /// ```
    fn to_direction_towards(&self, other: &Vec2) -> Option<SquareDirection> {
        self.towards(other).to_direction()
    }
}
