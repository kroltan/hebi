use bevy::prelude::*;
use bevy::core::FixedTimestep;

#[allow(unused)] mod colors;
#[allow(unused)] mod themes;

use themes::dracula as theme;

// World width in grid cells
const GRID_WIDTH: u32 = 29;

// World height in grid cells
const GRID_HEIGHT: u32 = 29;

// Pixel dimension of grid cell
const GRID_SCALE: u32 = 24;

// Pixel padding outside of grid
const GRID_PADDING: u32 = 24;

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("world_spawn", SystemStage::single(world_spawn.system()))
        .add_startup_stage("snake_spawn", SystemStage::single(snake_spawn.system()))
        .add_system(snake_movement_input.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.125))
                .with_system(snake_movement.system())
                .with_system(tick.system())
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource({
            let title = "Hebi".to_string();
            let width = (GRID_WIDTH * GRID_SCALE + GRID_PADDING * 2) as f32;
            let height = (GRID_HEIGHT * GRID_SCALE + GRID_PADDING * 2) as f32;
            println!(
                "Configuring window with a title of '{}', a width of {} pixels, and a height of {} pixels.",
                title, width, height
            );
            WindowDescriptor {
                title,
                width,
                height,
                resizable: false,
                ..Default::default()
            }
        })
        .insert_resource(ClearColor(Color::hex(theme::BACKGROUND).unwrap()))
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials::new(materials));
    commands.insert_resource(Clock::default());
}

fn grid_positioning(
    mut query: Query<(&GridPosition, &mut Transform)>,
) {
    for (grid_position, mut transform) in query.iter_mut() {
        assert!(grid_position.in_bounds());
        transform.translation = transform.translation.lerp(
            grid_to_vector(grid_position),
            match grid_position.t {
                Some(t) => t,
                None => 1.0,
            },
        );
    }
}

fn grid_to_vector(grid_position: &GridPosition) -> Vec3 {
    Vec3::new(
        (grid_position.x as f32 - GRID_WIDTH as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
        (grid_position.y as f32 - GRID_HEIGHT as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
        0.0,
    )
}

fn world_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.grid_background.clone(),
            sprite: Sprite::new(
                Vec2::new(
                    (GRID_WIDTH * GRID_SCALE) as f32,
                    (GRID_HEIGHT * GRID_SCALE) as f32
                )
            ),
            ..Default::default()
        });
}

fn snake_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
) {
    const DIRECTION: Direction = Direction::Up;
    const SEGMENTS: u32 = 7;
    let mut snake_head = SnakeHead::new(DIRECTION);
    let snake_head_position = GridPosition::center();
    let segment_direction = snake_head.direction.opposite().vec();
    for i in 1..SEGMENTS {
        snake_head.spawn_segment(&mut commands, &materials, GridPosition::new(
            ((segment_direction.x * (i as f32)) + snake_head_position.x as f32) as u32,
            ((segment_direction.y * (i as f32)) + snake_head_position.y as f32) as u32,
        ))
    }
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.snake.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SCALE as f32 * 0.875, GRID_SCALE as f32 * 0.875)),
            transform: Transform::from_translation(grid_to_vector(&snake_head_position)),
            ..Default::default()
        })
        .insert(snake_head_position)
        .insert(snake_head);
}

fn snake_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut snake_heads: Query<&mut SnakeHead>,
) {
    for mut snake_head in snake_heads.iter_mut() {
        let direction: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            snake_head.direction
        };
        if direction != snake_head.direction.opposite() {
            snake_head.next_direction = direction;
        }
    }
}

fn snake_movement(
    mut snake_heads: Query<(&mut SnakeHead, &mut GridPosition)>,
    mut grid_positions: Query<&mut GridPosition, Without<SnakeHead>>,
) {
    for (mut snake_head, mut grid_position) in snake_heads.iter_mut() {
        snake_head.direction = snake_head.next_direction;
        let direction_vector = snake_head.direction.vec();
        snake_head.update_segment_positions(&grid_position, &mut grid_positions);
        grid_position.x = (grid_position.x as f32 + direction_vector.x) as u32;
        grid_position.y = (grid_position.y as f32 + direction_vector.y) as u32;
    }
}

fn tick(
    mut clock: ResMut<Clock>
) {
    clock.tick();
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
    fn vec(&self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
            Self::Down => Vec2::new(0.0, -1.0),
            Self::Up => Vec2::new(0.0, 1.0),
        }
    }
}

struct SnakeHead {
    direction: Direction,
    next_direction: Direction,
    segments: Vec<Entity>,
}

struct SnakeHeads;


impl SnakeHead {
    fn new(direction: Direction) -> Self {
        SnakeHead {
            direction: direction,
            next_direction: direction,
            segments: Vec::new(),
        }
    }
    fn spawn_segment(
        &mut self,
        commands: &mut Commands,
        materials: &Res<Materials>,
        grid_position: GridPosition,
    ) {
        self.segments.push(commands
            .spawn_bundle(SpriteBundle {
                material: materials.snake.clone(),
                sprite: Sprite::new(Vec2::new(GRID_SCALE as f32 * 0.75, GRID_SCALE as f32 * 0.75)),
                transform: Transform::from_translation(grid_to_vector(&grid_position)),
                ..Default::default()
            })
            .insert(SnakeSegment)
            .insert(grid_position)
            .id()
        );
    }
    fn update_segment_positions(
        &mut self,
        head_position: &GridPosition,
        grid_positions: &mut Query<&mut GridPosition, Without<SnakeHead>>,
    ) {
        let mut new_segment_positions = Vec::<GridPosition>::new();
        for (i, _segment_position) in self.segments.iter().enumerate() {
            if i == 0 {
                new_segment_positions.push(head_position.clone());
                continue;
            }
            new_segment_positions.push((grid_positions.get_mut(*self.segments.get(i - 1).unwrap()).unwrap()).clone());
        }
        for (i, new_segment_position) in new_segment_positions.iter().enumerate() {
            let mut segment_position = grid_positions.get_mut(*self.segments.get(i).unwrap()).unwrap();
            segment_position.x = new_segment_position.x;
            segment_position.y = new_segment_position.y;
        }
    }
}

struct SnakeSegment;

#[derive(Default, Clone)]
struct GridPosition {
    x: u32,
    y: u32,
    t: Option<f32>,
}

impl GridPosition {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y, t: Some(0.375) }
    }
    fn center() -> Self {
        Self::new(
            (GRID_WIDTH as f32 / 2.0) as u32,
            (GRID_HEIGHT as f32 / 2.0) as u32,
        )
    }
    fn in_bounds(&self) -> bool {
        self.x < GRID_WIDTH && self.y < GRID_HEIGHT
    }
}

#[derive(Default)]
struct Clock {
    ticks: u32
}

impl Clock {
    fn tick(&mut self) -> u32 {
        self.ticks += 1;
        self.ticks
    }
}

struct Materials {
    grid_background: Handle<ColorMaterial>,
    snake: Handle<ColorMaterial>,
    food: Handle<ColorMaterial>,
}

impl Materials {
    fn new(mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        Materials {
            grid_background: materials.add(Color::hex(theme::GRID_BACKGROUND).unwrap().into()),
            snake: materials.add(Color::hex(theme::SNAKE).unwrap().into()),
            food: materials.add(Color::hex(theme::FOOD).unwrap().into()),
        }
    }
}