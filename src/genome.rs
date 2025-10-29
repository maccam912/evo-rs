use bevy::prelude::*;
use rand::Rng;
use crate::config::*;
use std::fmt;

/// Stack value types for the stack machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackValue {
    Float(f32),
    Bool(bool),
}

impl StackValue {
    pub fn as_float(&self) -> Option<f32> {
        match self {
            StackValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StackValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for StackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackValue::Float(val) => write!(f, "{:.2}", val),
            StackValue::Bool(val) => write!(f, "{}", val),
        }
    }
}

/// Word set for stack-based genome execution (Forth-like concatenative language)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Word {
    // Stack Manipulation
    Dup,          // ( a -- a a )
    Drop,         // ( a -- )
    Swap,         // ( a b -- b a )
    Over,         // ( a b -- a b a )
    Rot,          // ( a b c -- b c a )

    // Literals
    PushFloat(f32),  // ( -- f32 )
    PushBool(bool),  // ( -- bool )

    // Sensor Operations (push sensor values)
    SmellFront,   // ( -- f32 ) - Push front smell sensor distance
    SmellBack,    // ( -- f32 ) - Push back smell sensor distance
    SmellLeft,    // ( -- f32 ) - Push left smell sensor distance
    SmellRight,   // ( -- f32 ) - Push right smell sensor distance
    Energy,       // ( -- f32 ) - Push current energy

    // Arithmetic Operations
    Add,          // ( a b -- a+b )
    Sub,          // ( a b -- a-b )
    Mul,          // ( a b -- a*b )
    Div,          // ( a b -- a/b )

    // Comparison Operations
    Lt,           // ( a b -- bool ) - a < b
    Gt,           // ( a b -- bool ) - a > b
    Eq,           // ( a b -- bool ) - a == b

    // Logic Operations
    And,          // ( bool bool -- bool )
    Or,           // ( bool bool -- bool )
    Not,          // ( bool -- bool )

    // Control Flow
    If,           // ( bool -- ) - Begin conditional
    Then,         // ( -- ) - End conditional / else branch
    Else,         // ( -- ) - Start else branch

    // Labels (markers for jumps)
    Label0,       // ( -- ) - Label marker 0
    Label1,       // ( -- ) - Label marker 1
    Label2,       // ( -- ) - Label marker 2
    Label3,       // ( -- ) - Label marker 3

    // Jumps (jump to label position)
    Jump0,        // ( -- ) - Jump to Label0
    Jump1,        // ( -- ) - Jump to Label1
    Jump2,        // ( -- ) - Jump to Label2
    Jump3,        // ( -- ) - Jump to Label3

    // Movement Actions (consume stack values)
    MoveForward,  // ( f32 -- ) - Move forward by distance
    MoveBackward, // ( f32 -- ) - Move backward by distance
    TurnLeft,     // ( f32 -- ) - Turn left by degrees
    TurnRight,    // ( f32 -- ) - Turn right by degrees

    // Resource Actions
    Eat,          // ( -- ) - Try to eat nearby plant
    Split,        // ( -- ) - Reproduce

    // Special
    Nop,          // ( -- ) - No operation
}

impl Word {
    /// Generate a random word with reasonable parameters
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        // Weighted random: bias toward useful patterns
        let r = rng.gen_range(0..100);
        match r {
            // Sensors (20%)
            0..=4 => Word::SmellFront,
            5..=9 => Word::SmellBack,
            10..=14 => Word::SmellLeft,
            15..=19 => Word::SmellRight,

            // Literals (20%)
            20..=24 => Word::PushFloat(rng.gen_range(0.01..0.2)),
            25..=29 => Word::PushFloat(rng.gen_range(0.05..0.9)),  // For turns
            30..=34 => Word::PushFloat(rng.gen_range(50.0..500.0)), // For comparisons
            35..=39 => Word::PushBool(rng.gen_bool(0.5)),

            // Comparisons (15%)
            40..=44 => Word::Lt,
            45..=49 => Word::Gt,
            50..=54 => Word::Eq,

            // Control Flow (10%)
            55..=59 => Word::If,
            60..=62 => Word::Then,
            63..=64 => Word::Else,

            // Arithmetic (10%)
            65..=67 => Word::Add,
            68..=70 => Word::Sub,
            71..=72 => Word::Mul,
            73..=74 => Word::Div,

            // Movement (15%)
            75..=78 => Word::MoveForward,
            79..=80 => Word::MoveBackward,
            81..=84 => Word::TurnLeft,
            85..=88 => Word::TurnRight,

            // Actions (5%)
            89..=93 => Word::Eat,
            94..=95 => Word::Split,

            // Labels (3%)
            96 => [Word::Label0, Word::Label1, Word::Label2, Word::Label3][rng.gen_range(0..4)],
            97 => [Word::Label0, Word::Label1, Word::Label2, Word::Label3][rng.gen_range(0..4)],
            98 => [Word::Label0, Word::Label1, Word::Label2, Word::Label3][rng.gen_range(0..4)],

            // Jumps (3%)
            _ => [Word::Jump0, Word::Jump1, Word::Jump2, Word::Jump3, Word::Dup, Word::Swap, Word::Energy, Word::Nop][rng.gen_range(0..8)],
        }
    }

    /// Get the category of this word for color-coding
    pub fn category(&self) -> WordCategory {
        match self {
            Word::Dup | Word::Drop | Word::Swap | Word::Over | Word::Rot => WordCategory::Stack,
            Word::PushFloat(_) | Word::PushBool(_) | Word::SmellFront | Word::SmellBack
            | Word::SmellLeft | Word::SmellRight | Word::Energy => WordCategory::Sensor,
            Word::Add | Word::Sub | Word::Mul | Word::Div
            | Word::Lt | Word::Gt | Word::Eq | Word::And | Word::Or | Word::Not => WordCategory::Arithmetic,
            Word::If | Word::Then | Word::Else
            | Word::Label0 | Word::Label1 | Word::Label2 | Word::Label3
            | Word::Jump0 | Word::Jump1 | Word::Jump2 | Word::Jump3 => WordCategory::Control,
            Word::MoveForward | Word::MoveBackward | Word::TurnLeft | Word::TurnRight
            | Word::Eat | Word::Split => WordCategory::Action,
            Word::Nop => WordCategory::Special,
        }
    }

    /// Get the stack effect description for display
    pub fn stack_effect(&self) -> &'static str {
        match self {
            Word::Dup => "( a -- a a )",
            Word::Drop => "( a -- )",
            Word::Swap => "( a b -- b a )",
            Word::Over => "( a b -- a b a )",
            Word::Rot => "( a b c -- b c a )",
            Word::PushFloat(_) => "( -- f32 )",
            Word::PushBool(_) => "( -- bool )",
            Word::SmellFront | Word::SmellBack | Word::SmellLeft | Word::SmellRight | Word::Energy => "( -- f32 )",
            Word::Add | Word::Sub | Word::Mul | Word::Div => "( a b -- result )",
            Word::Lt | Word::Gt | Word::Eq => "( a b -- bool )",
            Word::And | Word::Or => "( bool bool -- bool )",
            Word::Not => "( bool -- bool )",
            Word::If => "( bool -- )",
            Word::Then | Word::Else => "( -- )",
            Word::Label0 | Word::Label1 | Word::Label2 | Word::Label3 => "( -- )",
            Word::Jump0 | Word::Jump1 | Word::Jump2 | Word::Jump3 => "( -- )",
            Word::MoveForward | Word::MoveBackward | Word::TurnLeft | Word::TurnRight => "( f32 -- )",
            Word::Eat | Word::Split => "( -- )",
            Word::Nop => "( -- )",
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Word::Dup => write!(f, "dup"),
            Word::Drop => write!(f, "drop"),
            Word::Swap => write!(f, "swap"),
            Word::Over => write!(f, "over"),
            Word::Rot => write!(f, "rot"),
            Word::PushFloat(val) => write!(f, "{:.1}", val),
            Word::PushBool(val) => write!(f, "{}", if *val { "true" } else { "false" }),
            Word::SmellFront => write!(f, "smell-front"),
            Word::SmellBack => write!(f, "smell-back"),
            Word::SmellLeft => write!(f, "smell-left"),
            Word::SmellRight => write!(f, "smell-right"),
            Word::Energy => write!(f, "energy"),
            Word::Add => write!(f, "+"),
            Word::Sub => write!(f, "-"),
            Word::Mul => write!(f, "*"),
            Word::Div => write!(f, "/"),
            Word::Lt => write!(f, "<"),
            Word::Gt => write!(f, ">"),
            Word::Eq => write!(f, "="),
            Word::And => write!(f, "and"),
            Word::Or => write!(f, "or"),
            Word::Not => write!(f, "not"),
            Word::If => write!(f, "if"),
            Word::Then => write!(f, "then"),
            Word::Else => write!(f, "else"),
            Word::Label0 => write!(f, "label0"),
            Word::Label1 => write!(f, "label1"),
            Word::Label2 => write!(f, "label2"),
            Word::Label3 => write!(f, "label3"),
            Word::Jump0 => write!(f, "jump0"),
            Word::Jump1 => write!(f, "jump1"),
            Word::Jump2 => write!(f, "jump2"),
            Word::Jump3 => write!(f, "jump3"),
            Word::MoveForward => write!(f, "move-forward"),
            Word::MoveBackward => write!(f, "move-backward"),
            Word::TurnLeft => write!(f, "turn-left"),
            Word::TurnRight => write!(f, "turn-right"),
            Word::Eat => write!(f, "eat"),
            Word::Split => write!(f, "split"),
            Word::Nop => write!(f, "nop"),
        }
    }
}

/// Category for color-coding words
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordCategory {
    Stack,       // Blue - Stack manipulation
    Sensor,      // Purple - Sensors and literals
    Arithmetic,  // Yellow - Arithmetic and logic
    Control,     // Orange - Control flow
    Action,      // Green - Movement and resource actions
    Special,     // Gray - Special operations
}

/// A genome is a sequence of words (Forth-like program)
#[derive(Component, Clone)]
pub struct Genome {
    pub words: Vec<Word>,
}

impl Genome {
    /// Create a new random genome
    pub fn random(length: usize) -> Self {
        let words = (0..length)
            .map(|_| Word::random())
            .collect();
        Self { words }
    }

    /// Create a mutated copy of this genome
    /// Each word has independent chances based on config rates
    pub fn mutate(&self) -> Self {
        let mut rng = rand::thread_rng();
        let mut new_words = Vec::new();

        for &word in &self.words {
            let should_delete = rng.gen_range(0..100) < DELETION_RATE;

            if should_delete {
                // Skip this word (delete it)
                continue;
            }

            let should_mutate = rng.gen_range(0..100) < MUTATION_RATE;
            let word_to_add = if should_mutate {
                Word::random()
            } else {
                word
            };

            new_words.push(word_to_add);

            // Check for duplication
            let should_duplicate = rng.gen_range(0..100) < DUPLICATION_RATE;
            if should_duplicate {
                new_words.push(word_to_add);
            }
        }

        // Ensure genome doesn't become empty
        if new_words.is_empty() {
            new_words.push(Word::random());
        }

        // Balance IF/THEN/ELSE
        Self::balance_control_flow(&mut new_words);

        Self { words: new_words }
    }

    /// Balance IF/THEN/ELSE to ensure valid control flow
    fn balance_control_flow(words: &mut Vec<Word>) {
        let mut if_count = 0;
        let mut then_count = 0;

        for word in words.iter() {
            match word {
                Word::If => if_count += 1,
                Word::Then => then_count += 1,
                Word::Else => {},
                _ => {}
            }
        }

        // Add missing THENs
        while then_count < if_count {
            words.push(Word::Then);
            then_count += 1;
        }

        // Remove extra THENs
        while then_count > if_count && then_count > 0 {
            if let Some(pos) = words.iter().rposition(|w| *w == Word::Then) {
                words.remove(pos);
                then_count -= 1;
            }
        }
    }
}

/// Control flow context for tracking IF/THEN/ELSE
#[derive(Debug, Clone)]
pub struct IfContext {
    pub if_position: usize,
    pub else_position: Option<usize>,
    pub then_position: Option<usize>,
    pub condition_result: bool,
    pub in_else_branch: bool,
}

/// Execution state for a genome
#[derive(Component)]
pub struct GenomeExecutor {
    pub instruction_pointer: usize,
    pub stack: Vec<StackValue>,
    pub instructions_executed_this_frame: u32,
    pub max_instructions_per_frame: u32,
    pub if_stack: Vec<IfContext>,
    pub jump_table: Vec<(usize, Option<usize>, usize)>, // (if_pos, else_pos, then_pos)
    pub label_table: [Option<usize>; 4], // Maps label index (0-3) to position in genome
}

impl GenomeExecutor {
    pub fn new(energy: u32) -> Self {
        Self {
            instruction_pointer: 0,
            stack: Vec::with_capacity(256),
            instructions_executed_this_frame: 0,
            max_instructions_per_frame: (energy * 1).min(MAX_INSTRUCTIONS_PER_FRAME),
            if_stack: Vec::new(),
            jump_table: Vec::new(),
            label_table: [None; 4],
        }
    }

    pub fn reset_for_frame(&mut self, energy: u32) {
        // DO NOT reset instruction_pointer (keep circular execution position)
        // DO NOT clear stack (persist values across frames)
        self.if_stack.clear();  // Clear control flow only
        self.instructions_executed_this_frame = 0;
        self.max_instructions_per_frame = (energy * 1).min(MAX_INSTRUCTIONS_PER_FRAME);
    }

    pub fn can_execute(&self) -> bool {
        self.instructions_executed_this_frame < self.max_instructions_per_frame
    }

    pub fn advance(&mut self, genome_len: usize) {
        self.instruction_pointer += 1;

        // Wrap around to start if we exceed genome length (circular execution)
        if self.instruction_pointer >= genome_len {
            self.instruction_pointer = 0;
        }

        self.instructions_executed_this_frame += 1;
    }

    /// Build jump table for IF/THEN/ELSE control flow
    pub fn build_jump_table(&mut self, genome: &Genome) {
        self.jump_table.clear();
        let mut if_stack: Vec<usize> = Vec::new();
        let mut else_positions: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        for (i, word) in genome.words.iter().enumerate() {
            match word {
                Word::If => {
                    if_stack.push(i);
                }
                Word::Else => {
                    if let Some(&if_pos) = if_stack.last() {
                        else_positions.insert(if_pos, i);
                    }
                }
                Word::Then => {
                    if let Some(if_pos) = if_stack.pop() {
                        let else_pos = else_positions.get(&if_pos).copied();
                        self.jump_table.push((if_pos, else_pos, i));
                    }
                }
                _ => {}
            }
        }
    }

    /// Build label table for jump targets
    pub fn build_label_table(&mut self, genome: &Genome) {
        // Reset all labels to None
        self.label_table = [None; 4];

        // Scan genome for label positions
        for (i, word) in genome.words.iter().enumerate() {
            match word {
                Word::Label0 => self.label_table[0] = Some(i),
                Word::Label1 => self.label_table[1] = Some(i),
                Word::Label2 => self.label_table[2] = Some(i),
                Word::Label3 => self.label_table[3] = Some(i),
                _ => {}
            }
        }
    }

    /// Push float to stack
    pub fn push_float(&mut self, value: f32) {
        if self.stack.len() < 256 {
            self.stack.push(StackValue::Float(value));
        }
    }

    /// Push bool to stack
    pub fn push_bool(&mut self, value: bool) {
        if self.stack.len() < 256 {
            self.stack.push(StackValue::Bool(value));
        }
    }

    /// Pop float from stack
    pub fn pop_float(&mut self) -> Option<f32> {
        self.stack.pop()?.as_float()
    }

    /// Pop bool from stack
    pub fn pop_bool(&mut self) -> Option<bool> {
        self.stack.pop()?.as_bool()
    }

    /// Pop any value from stack
    pub fn pop(&mut self) -> Option<StackValue> {
        self.stack.pop()
    }

    /// Peek at top of stack
    pub fn peek(&self) -> Option<&StackValue> {
        self.stack.last()
    }
}

/// Sensor data for an animal (4 directional smell sensors)
#[derive(Component, Default)]
pub struct Sensors {
    pub smell_front: Option<f32>,
    pub smell_back: Option<f32>,
    pub smell_left: Option<f32>,
    pub smell_right: Option<f32>,
}
