use std::ops::Range;

use crossterm::style::Color;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub unique_id: u16,

    pub x: i16,
    pub y: i16,
    pub denotation: char,
    pub color: Color,
    pub hp: i16,
    pub attack_skill: i16,
    pub damage_range: Range<i16>,

    pub defence_class: i16,
    pub attack_range: i16,

    pub attack_rate: i16,
    pub attack_cooldown: i16,

    pub movement_rate: i16,
    pub movement_cooldown: i16,

    pub is_attacking: bool,
}

#[derive(Debug, PartialEq)]
pub enum CommandState {
    Menu,
    MainGame,
    CharacterSelected(char),
    Chat,
}
