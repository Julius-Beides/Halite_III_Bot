extern crate rand;

use std::cell::RefCell;
use std::rc::Rc;
use hlt::ship::Ship;
use hlt::command::Command;
use hlt::log::Log;
use hlt::ShipId;
use hlt::direction::Direction;
use rand::Rng;
use hlt::game::Game;
use extended_map::ExtendedMap;

/* This is a more intelligent ship.
 * It has a command queue for the next few turns. */
pub struct ShipBot {
    pub ship_id: ShipId,
    movement_queue: Vec<Direction>,
    logger: Rc<RefCell<Log>>,
    not_moved: u32,
}

impl ShipBot {

    pub fn generate(ship_id: &ShipId, logger: Rc<RefCell<Log>>) -> ShipBot {
        ShipBot {
            ship_id: ship_id.clone(),
            movement_queue: Vec::new(),
            logger,
            not_moved: 0,
        }
    }

    /* Returns a queued action or
     * processes the AI to come up with actions.
     * Returns an Error if the ship doesn't exist anymore. */
    pub fn next_frame(&mut self, game: &Game, ex_map: &mut ExtendedMap) -> Result<Command, String> {
        // First, find out if the ship still exists.
        let hlt_ship: &Ship;
        match game.ships.get(&self.ship_id) {
            Some(ship) => hlt_ship = ship,
            None =>
                return Result::Err(format!("The ship {} doesn't exist anymore!", &self.ship_id.0))
        }

        // if queue empty
        if self.movement_queue.len() <= 0 {
            self.calculate_ai();
        }

        // Pop one action per round.
        let mut retry: Option<Direction> = None;
        let command = match self.movement_queue.pop() {
            // Try to move ship, but stay still if a collision would occur.
            // fixme Deadlock
            Some(direction) => {
                if ex_map.can_move_safely_then_reserve(&hlt_ship.position.directional_offset(direction)) {
                    hlt_ship.move_ship(direction)
                } else {
                    if direction != Direction::Still {
                        retry = Some(direction);
                        self.not_moved += 1;
                    }
                    hlt_ship.stay_still()
                }
            },
            None => { // Fail-safe: Stay still.
                self.logger.borrow_mut().log("ShipBot: The AI didn't add Actions!");
                hlt_ship.stay_still()
            }
        };
        // If movement failed, try it next turn.
        match retry {
            Some(dir) => self.movement_queue.push(dir),
            None => {}
        }

        // If not moved for to long, try some other random movement
        if self.not_moved >= 3 {
            self.not_moved = 0;
            self.queue_random_movement(2, 4);
        }

        return Result::Ok(command);
    }


    fn calculate_ai(&mut self) {
        const MAX_STEPS: i32 = 13;
        const MIN_STEPS: i32 = 11;
        self.queue_random_movement(MIN_STEPS, MAX_STEPS)
    }

    fn queue_random_movement(&mut self, min_steps: i32, max_steps: i32) {
        // Make sure to only move in one quadrant.
        // Don't let random movement cancel itself out.

        let vertical_direction =
            if rand::thread_rng().gen_bool(0.5) {
                Direction::North
            } else { Direction::South };

        let horizontal_direction =
            if rand::thread_rng().gen_bool(0.5) {
                Direction::East
            } else { Direction::West };

        let num_steps = rand::thread_rng().gen_range(min_steps,max_steps);
        let mut directions: Vec<Direction> = Vec::new();

        for _ in 0..num_steps {
            // Either go horizontally or vertically.
            if rand::thread_rng().gen_bool(0.5) {
                directions.push(vertical_direction)
            } else {
                directions.push(horizontal_direction)
            };
        }
        // and now backwards and collect stuff.
        for direction in directions.iter() {
            self.movement_queue.push(direction.clone());

            self.movement_queue.push(Direction::Still);
            self.movement_queue.push(Direction::Still);
            //self.movement_queue.push(ship.stay_still());
        }
        // First forwards.
        for direction in directions.iter().rev() {
            self.movement_queue.push(direction.invert_direction());
            self.movement_queue.push(Direction::Still);
        }
    }
}



