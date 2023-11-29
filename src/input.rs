use bevy::prelude::*;

use crate::InputDelay;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputBindings{
            up: KeyCode::Up,
            down: KeyCode::Down,
            right: KeyCode::Right,
            left: KeyCode::Left,

            q: KeyCode::Q,
            w: KeyCode::W,
            e: KeyCode::E,
            r: KeyCode::R,

            one: KeyCode::Key1,
            two: KeyCode::Key2,
            three: KeyCode::Key3,
            four: KeyCode::Key4,
            five: KeyCode::Key5,
            six: KeyCode::Key6,
        });
        app.insert_resource(LastAction{last: ActionType::Nothing});
        app.add_systems(Update, await_input);
    }
}

#[derive(PartialEq, Clone)]
pub enum ActionType{
    WalkUp,
    WalkLeft,
    WalkRight,
    WalkDown,
    QAbility,
    WAbility,
    EAbility,
    RAbility,
    OneItem,
    TwoItem,
    ThreeItem,
    FourItem,
    FiveItem,
    SixItem,
    Nothing,
}

#[derive(Resource)]
pub struct LastAction{
    pub last: ActionType
}

#[derive(Resource)]
struct InputBindings{
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    q: KeyCode,
    w: KeyCode,
    e: KeyCode,
    r: KeyCode,
    one: KeyCode,
    two: KeyCode,
    three: KeyCode,
    four: KeyCode,
    five: KeyCode,
    six: KeyCode,
}

fn await_input(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut delay: ResMut<InputDelay>,
    bindings: Res<InputBindings>,
    mut action: ResMut<LastAction>,
) {
    if !delay.time.finished() {
        delay.time.tick(time.delta());
    }
    if delay.time.finished() {
        let mut reset_queued = true;
        if input.pressed(bindings.up){
            action.last = ActionType::WalkUp;
        }
        else if input.pressed(bindings.down){
            action.last = ActionType::WalkDown;
        }
        else if input.pressed(bindings.left){
            action.last = ActionType::WalkLeft;
        }
        else if input.pressed(bindings.right){
            action.last = ActionType::WalkRight;
        }
        else if input.pressed(bindings.q){
            action.last = ActionType::QAbility;
        }
        else if input.pressed(bindings.w){
            action.last = ActionType::WAbility;
        }
        else if input.pressed(bindings.e){
            action.last = ActionType::EAbility;
        }
        else if input.pressed(bindings.r){
            action.last = ActionType::RAbility;
        }
        else if input.pressed(bindings.one){
            action.last = ActionType::OneItem;
        }
        else if input.pressed(bindings.two){
            action.last = ActionType::TwoItem;
        }
        else if input.pressed(bindings.three){
            action.last = ActionType::ThreeItem;
        }
        else if input.pressed(bindings.four){
            action.last = ActionType::FourItem;
        }
        else if input.pressed(bindings.five){
            action.last = ActionType::FiveItem;
        }
        else if input.pressed(bindings.six){
            action.last = ActionType::SixItem;
        }
        else { reset_queued = false;}
        if reset_queued {delay.time.reset();}
    }

}

pub fn direction(action: ActionType) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if action == ActionType::WalkUp {
        direction.y = 1.;
    }
    if action == ActionType::WalkDown {
        direction.y = -1.;
    }
    if action == ActionType::WalkRight {
        direction.x = 1.;
    }
    if action == ActionType::WalkLeft {
        direction.x = -1.;
    }
    direction
}
