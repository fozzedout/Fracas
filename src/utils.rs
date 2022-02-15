use std::time::SystemTime;
use crossterm::style::Color;

use crate::lib::*;

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


pub fn generate_barbarian(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'B',
        color: c,
        hp: 12,
        attack_skill: 3,
        defence_class: 9,
        attack_range: 1,
        damage_range: (1..7),
        attack_rate: 5,
        attack_cooldown: 5,
        movement_rate: 7,
        movement_cooldown: 7,
        is_attacking: false,
    }
}

pub fn generate_archer(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'A',
        color: c,
        hp: 6,
        attack_skill: 2,
        defence_class: 7,
        attack_range: 5,
        damage_range: (1..4),
        attack_rate: 10,
        attack_cooldown: 10,
        movement_rate: 13,
        movement_cooldown: 13,
        is_attacking: false,
    }
}

pub fn generate_giant(y: i16, c: Color) -> Character {
    let x = if c == Color::Green { 1 } else { 68 };

    Character {
        unique_id: fastrand::u16(0..65_000),
        x: x,
        y: y,
        denotation: 'G',
        color: c,
        hp: 30,
        attack_skill: 4,
        defence_class: 12,
        attack_range: 1,
        damage_range: (6..12),
        attack_rate: 15,
        attack_cooldown: 15,
        movement_rate: 30,
        movement_cooldown: 30,
        is_attacking: false,
    }
}
