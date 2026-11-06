use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operator {
    Delete,
    Change,
    Yank,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Motion {
    Word,
    WordEnd,
    BackWord,
    LineStart,
    FirstNonblank,
    LineEnd,
    Left,
    Right,
    Up,
    Down,
    ParagraphNext,
    ParagraphPrevious,
    FileStart,
    FileEnd,
    FileLine,
    WordBig,
    BackWordBig,
    WordEndBig,
    WordEndBackward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CharacterFind {
    pub character: char,
    pub backward: bool,
    pub till: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Motion(Motion),
    Word { around: bool },
    Paragraph { around: bool },
    Delimited { delimiter: char, around: bool },
    Lines,
    MatchingBracket,
    Find { find: CharacterFind, repeated: bool },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextCase {
    Lower,
    Upper,
    Toggle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Indentation {
    Indent,
    Outdent,
    Auto,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CursorScrollPosition {
    Center,
    Top,
    Bottom,
}
impl Indentation {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            ">" => Some(Self::Indent),
            "<" => Some(Self::Outdent),
            "=" => Some(Self::Auto),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Command {
    Join {
        count: usize,
    },
    Transform {
        case: TextCase,
        target: Target,
        count: usize,
    },
    Comment {
        target: Target,
        count: usize,
    },
    MatchBracket {
        operator: Option<Operator>,
    },
    Indent {
        kind: Indentation,
        count: usize,
    },
    Find {
        find: CharacterFind,
        operator: Option<Operator>,
        count: usize,
    },
    RepeatFind {
        reverse: bool,
        operator: Option<Operator>,
        count: usize,
    },
    SearchWord {
        backwards: bool,
    },
    RepeatSearch {
        reverse: bool,
    },
    SelectTextObject {
        target: Target,
        count: usize,
    },
    Edit {
        operator: Operator,
        target: Target,
        count: usize,
    },
    Move {
        motion: Motion,
        count: usize,
    },
    Paste {
        before: bool,
        count: usize,
    },
    Insert {
        append: bool,
    },
    DeleteCharacters {
        count: usize,
    },
    ScrollCursor(CursorScrollPosition),
    Repeat {
        count: Option<usize>,
    },
}
#[derive(Clone, Default)]
pub(super) struct InputState {
    count: usize,
    operator: Option<Operator>,
    operator_count: usize,
    around: Option<bool>,
    find: Option<(bool, bool)>,
    indentation: Option<Indentation>,
    file_start: bool,
    pending_g: bool,
    pending_z: bool,
    pending_case: Option<TextCase>,
    pending_comment: bool,
    visual_around: Option<bool>,
}
impl InputState {
    pub fn is_pending(&self) -> bool {
        self.count > 0
            || self.operator.is_some()
            || self.around.is_some()
            || self.find.is_some()
            || self.indentation.is_some()
            || self.file_start
            || self.pending_g
            || self.pending_z
            || self.pending_case.is_some()
            || self.pending_comment
            || self.visual_around.is_some()
    }
    pub fn waiting_for_character(&self) -> bool {
        self.find.is_some()
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn feed_visual(&mut self, key: &str) -> Option<Command> {
        if let Some(around) = self.visual_around.take() {
            let target = text_object(key, around);
            let count = self.count.max(1);
            self.clear();
            return target.map(|target| Command::SelectTextObject { target, count });
        }
        if matches!(key, "i" | "a") {
            self.visual_around = Some(key == "a");
            return None;
        }
        let command = self.feed(key);
        if self.indentation.is_some() && Indentation::from_key(key).is_some() {
            self.feed(key)
        } else {
            command
        }
    }
    pub fn feed(&mut self, key: &str) -> Option<Command> {
        if self.pending_z {
            let command = match key {
                "z" => Some(Command::ScrollCursor(CursorScrollPosition::Center)),
                "t" => Some(Command::ScrollCursor(CursorScrollPosition::Top)),
                "b" => Some(Command::ScrollCursor(CursorScrollPosition::Bottom)),
                _ => None,
            };
            self.clear();
            return command;
        }
        if let Some((backward, till)) = self.find {
            let mut characters = key.chars();
            let character = characters.next();
            let command = character
                .filter(|character| !character.is_control() && characters.next().is_none())
                .map(|character| Command::Find {
                    find: CharacterFind {
                        character,
                        backward,
                        till,
                    },
                    operator: self.operator,
                    count: self.count.max(1) * self.operator_count.max(1),
                });
            self.clear();
            return command;
        }
        if self.pending_g {
            self.pending_g = false;
            match key {
                "g" | "e" => {
                    self.file_start = true;
                    return self.feed(key);
                }
                "u" => {
                    self.pending_case = Some(TextCase::Lower);
                    return None;
                }
                "U" => {
                    self.pending_case = Some(TextCase::Upper);
                    return None;
                }
                "c" => {
                    self.pending_comment = true;
                    return None;
                }
                _ => {
                    self.clear();
                    return None;
                }
            }
        }
        if self.pending_case.is_some() || self.pending_comment {
            if matches!(key, "i" | "a") {
                self.around = Some(key == "a");
                return None;
            }
            let count = self.count.max(1) * self.operator_count.max(1);
            let target = if self.pending_comment && key == "c" {
                Some(Target::Lines)
            } else if let Some(around) = self.around {
                match key {
                    "w" => Some(Target::Word { around }),
                    "p" => Some(Target::Paragraph { around }),
                    "\"" | "'" => key
                        .chars()
                        .next()
                        .map(|delimiter| Target::Delimited { delimiter, around }),
                    "(" | ")" => Some(Target::Delimited {
                        delimiter: '(',
                        around,
                    }),
                    "[" | "]" => Some(Target::Delimited {
                        delimiter: '[',
                        around,
                    }),
                    "{" | "}" => Some(Target::Delimited {
                        delimiter: '{',
                        around,
                    }),
                    _ => None,
                }
            } else {
                let motion = match key {
                    "w" => Some(Motion::Word),
                    "W" => Some(Motion::WordBig),
                    "e" => Some(Motion::WordEnd),
                    "E" => Some(Motion::WordEndBig),
                    "b" => Some(Motion::BackWord),
                    "B" => Some(Motion::BackWordBig),
                    "$" => Some(Motion::LineEnd),
                    "0" => Some(Motion::LineStart),
                    "G" => Some(Motion::FileEnd),
                    _ => None,
                };
                motion.map(Target::Motion)
            };
            let command = target.map(|target| {
                if self.pending_comment {
                    Command::Comment { target, count }
                } else {
                    Command::Transform {
                        case: self.pending_case.unwrap_or(TextCase::Toggle),
                        target,
                        count,
                    }
                }
            });
            self.clear();
            return command;
        }
        if self.file_start {
            let command = if key == "g" {
                let count = self.count.max(1) * self.operator_count.max(1);
                let motion = if self.count > 0 || self.operator_count > 1 {
                    Motion::FileLine
                } else {
                    Motion::FileStart
                };
                Some(match self.operator {
                    Some(operator) => Command::Edit {
                        operator,
                        target: Target::Motion(motion),
                        count,
                    },
                    None => Command::Move { motion, count },
                })
            } else if key == "e" {
                let count = self.count.max(1) * self.operator_count.max(1);
                Some(match self.operator {
                    Some(operator) => Command::Edit {
                        operator,
                        target: Target::Motion(Motion::WordEndBackward),
                        count,
                    },
                    None => Command::Move {
                        motion: Motion::WordEndBackward,
                        count,
                    },
                })
            } else {
                None
            };
            self.clear();
            return command;
        }
        if let Some(around) = self.around {
            let target = match key {
                "w" => Target::Word { around },
                "p" => Target::Paragraph { around },
                "\"" | "'" => Target::Delimited {
                    delimiter: key.chars().next()?,
                    around,
                },
                "(" | ")" => Target::Delimited {
                    delimiter: '(',
                    around,
                },
                "[" | "]" => Target::Delimited {
                    delimiter: '[',
                    around,
                },
                "{" | "}" => Target::Delimited {
                    delimiter: '{',
                    around,
                },
                _ => {
                    self.clear();
                    return None;
                }
            };
            let command = self.operator.map(|operator| Command::Edit {
                operator,
                target,
                count: self.operator_count * self.count.max(1),
            });
            self.clear();
            return command;
        }
        if key.len() == 1
            && let Some(digit) = key
                .chars()
                .next()
                .and_then(|character| character.to_digit(10))
            && (digit != 0 || self.count > 0)
        {
            // Reject excessive counts before they can allocate huge paste strings or block input.
            if let Some(count) = self
                .count
                .checked_mul(10)
                .and_then(|count| count.checked_add(digit as usize))
                .filter(|count| *count <= 10_000 && *count <= 10_000 / self.operator_count.max(1))
            {
                self.count = count;
            } else {
                self.clear();
            }
            return None;
        }
        if let Some(pending) = self.indentation {
            let command =
                (Indentation::from_key(key) == Some(pending)).then_some(Command::Indent {
                    kind: pending,
                    count: self.operator_count.max(1) * self.count.max(1),
                });
            self.clear();
            return command;
        }
        if let Some(kind) = Indentation::from_key(key) {
            if self.operator.is_none() {
                self.indentation = Some(kind);
                self.operator_count = self.count.max(1);
                self.count = 0;
            } else {
                self.clear();
            }
            return None;
        }
        if matches!(key, "f" | "F" | "t" | "T") {
            self.find = Some((matches!(key, "F" | "T"), matches!(key, "t" | "T")));
            return None;
        }
        if matches!(key, ";" | ",") {
            let command = Command::RepeatFind {
                reverse: key == ",",
                operator: self.operator,
                count: self.count.max(1) * self.operator_count.max(1),
            };
            self.clear();
            return Some(command);
        }
        if matches!(key, "*" | "#") && self.operator.is_none() {
            self.clear();
            return Some(Command::SearchWord {
                backwards: key == "#",
            });
        }
        if matches!(key, "n" | "N") && self.operator.is_none() {
            self.clear();
            return Some(Command::RepeatSearch { reverse: key == "N" });
        }
        let operator = match key {
            "d" => Some(Operator::Delete),
            "c" => Some(Operator::Change),
            "y" => Some(Operator::Yank),
            _ => None,
        };
        if self.operator.is_some() && matches!(key, "i" | "a") {
            self.around = Some(key == "a");
            return None;
        }
        if let Some(operator) = operator {
            if let Some(pending) = self.operator {
                let command = (pending == operator).then_some(Command::Edit {
                    operator,
                    target: Target::Lines,
                    count: self.operator_count * self.count.max(1),
                });
                self.clear();
                return command;
            }
            self.operator = Some(operator);
            self.operator_count = self.count.max(1);
            self.count = 0;
            return None;
        }
        // A count on % means a file percentage in Vim, not repeated bracket matching.
        if key == "g" {
            self.pending_g = true;
            return None;
        }
        if key == "z" && self.operator.is_none() {
            self.pending_z = true;
            return None;
        }
        if key == "~" {
            let count = self.count.max(1);
            self.clear();
            return Some(Command::Transform {
                case: TextCase::Toggle,
                target: Target::Motion(Motion::Right),
                count,
            });
        }
        if key == "%" {
            let command =
                (self.count == 0 && self.operator_count <= 1).then_some(Command::MatchBracket {
                    operator: self.operator,
                });
            self.clear();
            return command;
        }
        let count = self.count.max(1) * self.operator_count.max(1);
        let motion = match key {
            "w" => Some(Motion::Word),
            "W" => Some(Motion::WordBig),
            "e" => Some(Motion::WordEnd),
            "E" => Some(Motion::WordEndBig),
            "b" => Some(Motion::BackWord),
            "B" => Some(Motion::BackWordBig),
            "0" => Some(Motion::LineStart),
            "^" => Some(Motion::FirstNonblank),
            "$" => Some(Motion::LineEnd),
            "h" => Some(Motion::Left),
            "l" => Some(Motion::Right),
            "j" => Some(Motion::Down),
            "k" => Some(Motion::Up),
            "}" => Some(Motion::ParagraphNext),
            "{" => Some(Motion::ParagraphPrevious),
            "G" => Some(if self.count > 0 || self.operator_count > 1 {
                Motion::FileLine
            } else {
                Motion::FileEnd
            }),
            _ => None,
        };
        let command = if let Some(motion) = motion {
            Some(match self.operator {
                Some(operator) => Command::Edit {
                    operator,
                    target: Target::Motion(motion),
                    count,
                },
                None => Command::Move { motion, count },
            })
        } else if self.operator.is_none() {
            match key {
                "p" | "P" => Some(Command::Paste {
                    before: key == "P",
                    count,
                }),
                "i" | "a" => Some(Command::Insert { append: key == "a" }),
                "x" => Some(Command::DeleteCharacters { count }),
                "J" => Some(Command::Join {
                    count: count.max(2),
                }),
                "." => Some(Command::Repeat {
                    count: (self.count > 0).then_some(count),
                }),
                _ => None,
            }
        } else {
            None
        };
        self.clear();
        command
    }
}

fn text_object(key: &str, around: bool) -> Option<Target> {
    Some(match key {
        "w" => Target::Word { around },
        "p" => Target::Paragraph { around },
        "\"" | "'" => Target::Delimited {
            delimiter: key.chars().next()?,
            around,
        },
        "(" | ")" => Target::Delimited {
            delimiter: '(',
            around,
        },
        "[" | "]" => Target::Delimited {
            delimiter: '[',
            around,
        },
        "{" | "}" => Target::Delimited {
            delimiter: '{',
            around,
        },
        _ => return None,
    })
}

pub(super) fn find_word_match(
    text: &str,
    query: &str,
    offset: usize,
    backwards: bool,
    inclusive: bool,
    excluded: Option<Range<usize>>,
) -> Option<Range<usize>> {
    if query.is_empty() {
        return None;
    }
    let matches = word_matches(text, query)
        .into_iter()
        .filter(|range| {
            excluded
                .as_ref()
                .is_none_or(|excluded| excluded.start != range.start || excluded.end != range.end)
        })
        .collect::<Vec<_>>();
    if backwards {
        matches
            .iter()
            .rev()
            .find(|range| {
                if inclusive {
                    range.start <= offset
                } else {
                    range.start < offset
                }
            })
            .or_else(|| matches.last())
            .cloned()
    } else {
        matches
            .iter()
            .find(|range| {
                if inclusive {
                    range.start >= offset
                } else {
                    range.start > offset
                }
            })
            .or_else(|| matches.first())
            .cloned()
    }
}

pub(super) fn word_matches(text: &str, query: &str) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    text
        .match_indices(query)
        .filter_map(|(start, matched)| {
            let end = start + matched.len();
            let left_is_word = text
                .get(..start)
                .and_then(|prefix| prefix.chars().next_back())
                .is_some_and(|character| kind(character) == 1);
            let right_is_word = text
                .get(end..)
                .and_then(|suffix| suffix.chars().next())
                .is_some_and(|character| kind(character) == 1);
            (!left_is_word && !right_is_word).then_some(start..end)
        })
        .collect()
}

fn kind(character: char) -> u8 {
    if character.is_whitespace() {
        0
    } else if character.is_alphanumeric() || character == '_' {
        1
    } else {
        2
    }
}
fn spans(text: &str) -> Vec<(Range<usize>, u8)> {
    let mut result: Vec<(Range<usize>, u8)> = Vec::new();
    for (offset, character) in text.char_indices() {
        let class = kind(character);
        if let Some((range, previous)) = result.last_mut()
            && *previous == class
        {
            range.end = offset + character.len_utf8();
        } else {
            result.push((offset..offset + character.len_utf8(), class));
        }
    }
    result
}
pub(super) fn next(text: &str, offset: usize) -> usize {
    text.get(offset..)
        .and_then(|tail| tail.chars().next())
        .map_or(offset, |character| offset + character.len_utf8())
}
pub(super) fn previous(text: &str, offset: usize) -> usize {
    text.get(..offset)
        .and_then(|head| head.char_indices().next_back())
        .map_or(0, |(offset, _)| offset)
}
pub(super) fn bracket_at_or_after(text: &str, offset: usize) -> Option<usize> {
    text.get(offset..)?
        .chars()
        .take_while(|character| *character != '\n')
        .scan(offset, |position, character| {
            let current = *position;
            *position += character.len_utf8();
            Some((current, character))
        })
        .find_map(|(position, character)| "()[]{}".contains(character).then_some(position))
}

pub(super) fn matching_bracket(text: &str, offset: usize) -> Option<usize> {
    let bracket = bracket_at_or_after(text, offset)?;
    let character = text.get(bracket..)?.chars().next()?;
    let (opening, closing, backward) = match character {
        '(' | ')' => ('(', ')', character == ')'),
        '[' | ']' => ('[', ']', character == ']'),
        '{' | '}' => ('{', '}', character == '}'),
        _ => return None,
    };
    let mut quoted = Vec::new();
    let mut skip_until = 0;
    for (position, character) in text.char_indices() {
        if position < skip_until {
            continue;
        }
        if matches!(character, '\'' | '"') {
            let end = line_end(text, position);
            let start = next(text, position);
            let mut escaped = false;
            let closing = text
                .get(start..end)?
                .char_indices()
                .find_map(|(relative, current)| {
                    let closes = current == character && !escaped;
                    escaped = current == '\\' && !escaped;
                    closes.then_some(start + relative + current.len_utf8())
                });
            if let Some(end) = closing {
                skip_until = end;
                quoted.push(position..end);
            }
        }
    }
    let quote_at = |position| {
        let index = quoted.partition_point(|range| range.start <= position);
        index
            .checked_sub(1)
            .and_then(|index| quoted.get(index))
            .filter(|range| range.contains(&position))
    };
    let active = quote_at(bracket).cloned().unwrap_or(0..text.len());
    let candidates = text
        .get(active.clone())?
        .char_indices()
        .map(|(index, character)| (active.start + index, character));
    let mut candidates: Box<dyn Iterator<Item = (usize, char)>> = if backward {
        Box::new(candidates.rev())
    } else {
        Box::new(candidates)
    };
    let mut depth = 0usize;
    candidates.find_map(|(position, character)| {
        if (backward && position > bracket)
            || (!backward && position < bracket)
            || quote_at(position).is_some_and(|quote| !quote.contains(&bracket))
        {
            return None;
        }
        if character == if backward { closing } else { opening } {
            depth += 1;
        } else if character == if backward { opening } else { closing } {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(position);
            }
        }
        None
    })
}

pub(super) fn find_character(
    text: &str,
    offset: usize,
    find: CharacterFind,
    count: usize,
    repeated: bool,
) -> Option<usize> {
    if count == 0 || find.character.is_control() {
        return None;
    }
    let start = text
        .get(..offset)?
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let end = line_end(text, offset);
    let matching = |character: &char| *character == find.character;
    let mut matches: Box<dyn Iterator<Item = usize> + '_> =
        if find.backward {
            Box::new(text.get(start..offset)?.char_indices().rev().filter_map(
                move |(index, character)| matching(&character).then_some(start + index),
            ))
        } else {
            let start = next(text, offset);
            Box::new(
                text.get(start..end)?
                    .char_indices()
                    .filter_map(move |(index, character)| {
                        matching(&character).then_some(start + index)
                    }),
            )
        };
    let matched = matches.nth(count - 1)?;
    let destination = |matched| {
        if find.till {
            if find.backward {
                next(text, matched)
            } else {
                previous(text, matched)
            }
        } else {
            matched
        }
    };
    let target = destination(matched);
    // Repeating t/T must pass an adjacent match that would leave the caret in place.
    if repeated && target == offset {
        matches.next().map(destination)
    } else {
        Some(target)
    }
}

fn find_range(
    text: &str,
    offset: usize,
    find: CharacterFind,
    count: usize,
    repeated: bool,
) -> Option<(Range<usize>, bool)> {
    let destination = find_character(text, offset, find, count, repeated)?;
    Some((
        if find.backward {
            destination..offset
        } else {
            offset..next(text, destination)
        },
        false,
    ))
}

pub(super) struct JoinedLines {
    pub range: Range<usize>,
    pub text: String,
    pub cursor: usize,
}

pub(super) fn join_lines(text: &str, first: usize, last: usize) -> Option<JoinedLines> {
    let starts = line_starts(text);
    let start = *starts.get(first)?;
    let last = last.min(starts.len().checked_sub(1)?);
    if last <= first {
        return None;
    }
    let end = line_end(text, *starts.get(last)?);
    let mut joined = text.get(start..line_end(text, start))?.to_string();
    let mut cursor = joined.len();
    for row in first + 1..=last {
        let start = *starts.get(row)?;
        let line = text
            .get(start..line_end(text, start))?
            .trim_start_matches([' ', '\t']);
        cursor = joined.len();
        if !joined.is_empty()
            && !joined.chars().next_back().is_some_and(char::is_whitespace)
            && !line.is_empty()
            && !line.starts_with(')')
        {
            joined.push(' ');
        }
        joined.push_str(line);
    }
    Some(JoinedLines {
        range: start..end,
        text: joined,
        cursor,
    })
}

fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect()
}
fn row(starts: &[usize], offset: usize) -> usize {
    starts
        .partition_point(|start| *start <= offset)
        .saturating_sub(1)
}
fn line_end(text: &str, start: usize) -> usize {
    text.get(start..)
        .and_then(|tail| tail.find('\n'))
        .map_or(text.len(), |length| start + length)
}
pub(super) fn normal_offset(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    let start = text
        .get(..offset)
        .and_then(|head| head.rfind('\n'))
        .map_or(0, |newline| newline + 1);
    if offset == line_end(text, start) && offset > start {
        previous(text, offset)
    } else {
        offset
    }
}
fn blank(text: &str, starts: &[usize], index: usize) -> bool {
    starts
        .get(index)
        .and_then(|start| text.get(*start..line_end(text, *start)))
        .is_none_or(|line| line.trim().is_empty())
}
pub(super) fn motion(text: &str, offset: usize, motion: Motion, count: usize) -> usize {
    let starts = line_starts(text);
    let current_row = row(&starts, offset);
    let start = starts.get(current_row).copied().unwrap_or(0);
    let end = line_end(text, start);
    match motion {
        Motion::Word
        | Motion::BackWord
        | Motion::WordEnd
        | Motion::WordBig
        | Motion::BackWordBig
        | Motion::WordEndBig
        | Motion::WordEndBackward => {
            let words = if matches!(
                motion,
                Motion::WordBig
                    | Motion::BackWordBig
                    | Motion::WordEndBig
                    | Motion::WordEndBackward
            ) {
                let mut ranges = Vec::new();
                let mut start = None;
                for (offset, character) in text.char_indices() {
                    if character.is_whitespace() {
                        if let Some(start) = start.take() {
                            ranges.push(start..offset);
                        }
                    } else if start.is_none() {
                        start = Some(offset);
                    }
                }
                if let Some(start) = start {
                    ranges.push(start..text.len());
                }
                ranges
            } else {
                spans(text)
                    .into_iter()
                    .filter(|(_, class)| *class != 0)
                    .map(|(range, _)| range)
                    .collect::<Vec<_>>()
            };
            let mut destination = offset;
            for _ in 0..count {
                destination = match motion {
                    Motion::Word | Motion::WordBig => words
                        .iter()
                        .find(|range| range.start > destination)
                        .map_or(text.len(), |range| range.start),
                    Motion::BackWord | Motion::BackWordBig => words
                        .iter()
                        .rev()
                        .find(|range| range.start < destination)
                        .map_or(0, |range| range.start),
                    Motion::WordEndBackward => words
                        .iter()
                        .rev()
                        .find(|range| previous(text, range.end) < destination)
                        .map_or(0, |range| previous(text, range.end)),
                    _ => words
                        .iter()
                        .find(|range| previous(text, range.end) > destination)
                        .map_or(previous(text, text.len()), |range| {
                            previous(text, range.end)
                        }),
                };
            }
            destination
        }
        Motion::Left | Motion::Right => {
            let mut destination = offset;
            for _ in 0..count {
                destination = if motion == Motion::Left {
                    previous(text, destination).max(start)
                } else {
                    next(text, destination).min(end)
                };
            }
            destination
        }
        Motion::FileStart => 0,
        Motion::FileEnd => normal_offset(text, text.len()),
        Motion::FileLine => normal_offset(
            text,
            starts
                .get(count.saturating_sub(1))
                .copied()
                .unwrap_or(text.len()),
        ),
        Motion::LineStart => start,
        Motion::FirstNonblank => {
            start
                + text
                    .get(start..end)
                    .map_or(0, |line| line.len() - line.trim_start().len())
        }
        Motion::LineEnd => {
            let start = starts
                .get(current_row.saturating_add(count - 1))
                .copied()
                .unwrap_or(text.len());
            let end = line_end(text, start);
            if end > start {
                previous(text, end)
            } else {
                start
            }
        }
        Motion::Up | Motion::Down => {
            let target_row = if motion == Motion::Up {
                current_row.saturating_sub(count)
            } else {
                current_row
                    .saturating_add(count)
                    .min(starts.len().saturating_sub(1))
            };
            let target_start = starts.get(target_row).copied().unwrap_or(0);
            let mut target =
                (target_start + offset.saturating_sub(start)).min(line_end(text, target_start));
            while !text.is_char_boundary(target) {
                target = target.saturating_sub(1);
            }
            target
        }
        Motion::ParagraphNext | Motion::ParagraphPrevious => {
            let mut target_row = current_row;
            for _ in 0..count {
                if motion == Motion::ParagraphNext {
                    while target_row + 1 < starts.len() && blank(text, &starts, target_row) {
                        target_row += 1;
                    }
                    while target_row + 1 < starts.len() && !blank(text, &starts, target_row) {
                        target_row += 1;
                    }
                } else {
                    while target_row > 0 && blank(text, &starts, target_row) {
                        target_row -= 1;
                    }
                    while target_row > 0 && !blank(text, &starts, target_row) {
                        target_row -= 1;
                    }
                }
            }
            if motion == Motion::ParagraphNext && !blank(text, &starts, target_row) {
                text.len()
            } else {
                starts.get(target_row).copied().unwrap_or(text.len())
            }
        }
    }
}
fn unescaped_quotes(text: &str, delimiter: char) -> Vec<usize> {
    let mut escaped = false;
    let mut quotes = Vec::new();
    for (offset, character) in text.char_indices() {
        if character == '\\' {
            escaped = !escaped;
        } else {
            if character == delimiter && !escaped {
                quotes.push(offset);
            }
            escaped = false;
        }
    }
    quotes
}

fn delimited_range(
    text: &str,
    offset: usize,
    delimiter: char,
    around: bool,
    count: usize,
) -> Option<(Range<usize>, bool)> {
    let line_start = text
        .get(..offset)?
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let current_line_end = line_end(text, line_start);
    if matches!(delimiter, '\'' | '"') {
        let quotes = unescaped_quotes(text.get(line_start..current_line_end)?, delimiter);
        let relative = offset - line_start;
        let (first, last) = if let Some(index) = quotes.iter().position(|quote| *quote == relative)
        {
            if index % 2 == 1 {
                (*quotes.get(index - 1)?, relative)
            } else {
                (relative, *quotes.get(index + 1)?)
            }
        } else if let Some(index) = quotes.iter().rposition(|quote| *quote < relative) {
            (*quotes.get(index)?, *quotes.get(index + 1)?)
        } else {
            (*quotes.first()?, *quotes.get(1)?)
        };
        let mut range = if around || count > 1 {
            line_start + first..line_start + last + 1
        } else {
            line_start + first + 1..line_start + last
        };
        if around {
            let trailing = text
                .get(range.end..current_line_end)?
                .chars()
                .take_while(|character| matches!(character, ' ' | '\t'))
                .map(char::len_utf8)
                .sum::<usize>();
            if trailing > 0 {
                range.end += trailing;
            } else {
                let leading = text
                    .get(line_start..range.start)?
                    .chars()
                    .rev()
                    .take_while(|character| matches!(character, ' ' | '\t'))
                    .map(char::len_utf8)
                    .sum::<usize>();
                range.start -= leading;
            }
        }
        return Some((range, false));
    }
    let closing = match delimiter {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let mut openings = Vec::new();
    let mut pairs = Vec::new();
    let mut skip_until = None;
    for (position, character) in text.char_indices() {
        if skip_until.is_some_and(|end| position <= end) {
            continue;
        }
        if matches!(character, '\'' | '"') {
            // Only skip a complete same-line string: an unmatched apostrophe may be a Rust lifetime.
            let end = line_end(text, position);
            if let Some(relative) = unescaped_quotes(text.get(position..end)?, character).get(1) {
                skip_until = Some(position + relative);
                continue;
            }
        }
        if character == delimiter {
            openings.push(position);
        } else if character == closing
            && let Some(first) = openings.pop()
        {
            pairs.push(first..position);
        }
    }
    let innermost = pairs
        .iter()
        .filter(|pair| pair.start <= offset && offset <= pair.end)
        .min_by_key(|pair| pair.len())
        .or_else(|| {
            pairs
                .iter()
                .filter(|pair| pair.start >= offset && pair.start <= current_line_end)
                .min_by_key(|pair| pair.start)
        })?;
    let mut enclosing = pairs
        .iter()
        .filter(|pair| pair.start <= innermost.start && pair.end >= innermost.end)
        .collect::<Vec<_>>();
    enclosing.sort_by_key(|pair| pair.len());
    let pair = enclosing.get(count.checked_sub(1)?)?;
    if around {
        return Some((pair.start..pair.end + 1, false));
    }
    let mut start = pair.start + 1;
    let mut end = pair.end;
    let opening_line_end = line_end(text, start);
    if opening_line_end < end && text.get(start..opening_line_end)?.trim().is_empty() {
        start = opening_line_end + 1;
    }
    let closing_line_start = text
        .get(..end)?
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    if closing_line_start >= start && text.get(closing_line_start..end)?.trim().is_empty() {
        end = closing_line_start;
    }
    let linewise = start != pair.start + 1 && end == closing_line_start;
    Some((start..end, linewise))
}

pub(super) fn resolve_range(
    text: &str,
    offset: usize,
    operator: Operator,
    target: Target,
    count: usize,
) -> Option<(Range<usize>, bool)> {
    match target {
        Target::MatchingBracket => {
            let destination = matching_bracket(text, offset)?;
            Some((
                offset.min(destination)..next(text, offset.max(destination)),
                false,
            ))
        }
        Target::Find { find, repeated } => find_range(text, offset, find, count, repeated),
        Target::Delimited { delimiter, around } => {
            delimited_range(text, offset, delimiter, around, count)
        }
        _ => Some(edit_range(text, offset, operator, target, count)),
    }
}

pub(super) fn edit_range(
    text: &str,
    offset: usize,
    operator: Operator,
    target: Target,
    count: usize,
) -> (Range<usize>, bool) {
    match target {
        Target::MatchingBracket => {
            resolve_range(text, offset, operator, target, count).unwrap_or((offset..offset, false))
        }
        Target::Find { find, repeated } => {
            find_range(text, offset, find, count, repeated).unwrap_or((offset..offset, false))
        }
        Target::Delimited { delimiter, around } => {
            delimited_range(text, offset, delimiter, around, count)
                .unwrap_or((offset..offset, false))
        }
        Target::Motion(direction) => {
            if matches!(direction, Motion::FileStart | Motion::FileEnd | Motion::FileLine) {
                let starts = line_starts(text);
                let destination = motion(text, offset, direction, count);
                let first = row(&starts, offset).min(row(&starts, destination));
                let last = row(&starts, offset).max(row(&starts, destination));
                return (
                    starts.get(first).copied().unwrap_or(0)
                        ..starts.get(last + 1).copied().unwrap_or(text.len()),
                    true,
                );
            }
            if matches!(direction, Motion::Up | Motion::Down) {
                let starts = line_starts(text);
                let first = row(&starts, offset);
                let last = row(&starts, motion(text, offset, direction, count));
                return (
                    starts.get(first.min(last)).copied().unwrap_or(0)
                        ..starts
                            .get(first.max(last) + 1)
                            .copied()
                            .unwrap_or(text.len()),
                    true,
                );
            }
            let destination = if operator == Operator::Change
                && direction == Motion::Word
                && text
                    .get(offset..)
                    .and_then(|tail| tail.chars().next())
                    .is_some_and(|character| !character.is_whitespace())
            {
                spans(text)
                    .into_iter()
                    .filter(|(range, class)| *class != 0 && range.end > offset)
                    .nth(count - 1)
                    .map_or(text.len(), |(range, _)| range.end)
            } else {
                let destination = motion(text, offset, direction, count);
                if direction == Motion::Word && count == 1 && offset < line_end(text, offset) {
                    destination.min(line_end(text, offset))
                } else if matches!(
                    direction,
                    Motion::WordEnd | Motion::WordEndBig | Motion::LineEnd
                ) && text
                    .get(destination..)
                    .and_then(|tail| tail.chars().next())
                    .is_some_and(|character| character != '\n')
                {
                    next(text, destination)
                } else {
                    destination
                }
            };
            (offset.min(destination)..offset.max(destination), false)
        }
        Target::Lines => {
            let starts = line_starts(text);
            let current_row = row(&starts, offset);
            (
                starts.get(current_row).copied().unwrap_or(0)
                    ..starts
                        .get(current_row + count)
                        .copied()
                        .unwrap_or(text.len()),
                true,
            )
        }
        Target::Word { around } => {
            let parts = spans(text);
            let index = parts
                .iter()
                .position(|(range, _)| range.contains(&offset))
                .unwrap_or(parts.len().saturating_sub(1));
            let Some((first, class)) = parts.get(index) else {
                return (offset..offset, false);
            };
            let mut range = first.clone();
            if !around {
                range.end = parts
                    .get(index.saturating_add(count - 1))
                    .or_else(|| parts.last())
                    .map_or(range.end, |(range, _)| range.end);
                return (range, false);
            }
            let mut words = 0;
            for (part, class) in parts.iter().skip(index) {
                range.end = part.end;
                if *class != 0 {
                    words += 1;
                }
                if words == count {
                    break;
                }
            }
            if *class != 0 {
                let trailing = text.get(range.end..).map_or(0, |tail| {
                    tail.chars()
                        .take_while(|character| character.is_whitespace() && *character != '\n')
                        .map(char::len_utf8)
                        .sum()
                });
                if trailing > 0 {
                    range.end += trailing;
                } else if let Some((leading, 0)) =
                    index.checked_sub(1).and_then(|index| parts.get(index))
                {
                    range.start = text
                        .get(leading.clone())
                        .and_then(|space| space.rfind('\n'))
                        .map_or(leading.start, |newline| leading.start + newline + 1);
                }
            }
            (range, false)
        }
        Target::Paragraph { around } => {
            let starts = line_starts(text);
            let mut first = row(&starts, offset);
            let mut last = first;
            let is_blank = blank(text, &starts, first);
            while first > 0 && blank(text, &starts, first - 1) == is_blank {
                first -= 1;
            }
            while last + 1 < starts.len() && blank(text, &starts, last + 1) == is_blank {
                last += 1;
            }
            if around && is_blank {
                if last + 1 < starts.len() {
                    last += 1;
                }
                while last + 1 < starts.len() && !blank(text, &starts, last + 1) {
                    last += 1;
                }
            }
            for _ in 1..count {
                if around {
                    while last + 1 < starts.len() && blank(text, &starts, last + 1) {
                        last += 1;
                    }
                }
                if last + 1 < starts.len() {
                    last += 1;
                }
                let next_is_blank = blank(text, &starts, last);
                while last + 1 < starts.len() && blank(text, &starts, last + 1) == next_is_blank {
                    last += 1;
                }
            }
            if around && !is_blank {
                let old_last = last;
                while last + 1 < starts.len() && blank(text, &starts, last + 1) {
                    last += 1;
                }
                if old_last == last {
                    while first > 0 && blank(text, &starts, first - 1) {
                        first -= 1;
                    }
                }
            }
            (
                starts.get(first).copied().unwrap_or(0)
                    ..starts.get(last + 1).copied().unwrap_or(text.len()),
                true,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(keys: &[&str]) -> Option<Command> {
        let mut state = InputState::default();
        keys.iter().filter_map(|key| state.feed(key)).last()
    }
    #[test]
    fn file_start_and_end_motions_parse_with_operators_and_counts() {
        assert_eq!(
            parse(&["g", "g"]),
            Some(Command::Move {
                motion: Motion::FileStart,
                count: 1
            })
        );
        assert_eq!(
            parse(&["d", "g", "g"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Motion(Motion::FileStart),
                count: 1
            })
        );
        assert_eq!(
            parse(&["2", "d", "g", "g"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Motion(Motion::FileStart),
                count: 2
            })
        );
        assert_eq!(
            parse(&["d", "G"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Motion(Motion::FileEnd),
                count: 1
            })
        );
        assert_eq!(
            parse(&["2", "d", "G"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Motion(Motion::FileEnd),
                count: 2
            })
        );
        assert_eq!(
            parse(&["G"]),
            Some(Command::Move {
                motion: Motion::FileEnd,
                count: 1
            })
        );
        assert_eq!(parse(&["d", "g", "q"]), None);
    }

    #[test]
    fn joins_parse_counts_and_leave_operator_pending_commands_safe() {
        assert_eq!(parse(&["J"]), Some(Command::Join { count: 2 }));
        assert_eq!(parse(&["1", "J"]), Some(Command::Join { count: 2 }));
        assert_eq!(parse(&["3", "J"]), Some(Command::Join { count: 3 }));
        assert_eq!(parse(&["d", "J"]), None);
        assert_eq!(
            parse(&["f", "J"]),
            Some(Command::Find {
                find: CharacterFind {
                    character: 'J',
                    backward: false,
                    till: false
                },
                operator: None,
                count: 1
            })
        );
    }

    #[test]
    fn joins_handle_indentation_existing_spaces_blank_lines_and_utf8() {
        for (text, last, expected, cursor) in [
            ("one\n  two", 1, "one two", 3),
            ("one  \n\ttwo", 1, "one  two", 5),
            ("one\n   ", 1, "one", 3),
            ("\ntwo", 1, "two", 0),
            ("one\n\n two", 2, "one two", 3),
            ("one\ntwo\n", 2, "one two", 7),
            ("one\n  )", 1, "one)", 3),
            ("界\n  字", 1, "界 字", 3),
            ("one\ntwo\nthree", 99, "one two three", 7),
        ] {
            let joined = join_lines(text, 0, last).expect("lines should join");
            assert_eq!(joined.range, 0..text.len());
            assert_eq!(joined.text, expected);
            assert_eq!(joined.cursor, cursor);
        }
        let joined = join_lines("head\none\ntwo\ntail", 1, 2).expect("middle lines should join");
        assert_eq!(joined.range, 5..12);
        assert_eq!(joined.text, "one two");
        assert!(join_lines("one", 0, 1).is_none());
        assert!(join_lines("one", 1, 2).is_none());
        assert!(join_lines("", 0, 1).is_none());
    }

    #[test]
    fn matching_brackets_parse_without_treating_counts_as_repeated_jumps() {
        assert_eq!(
            parse(&["%"]),
            Some(Command::MatchBracket { operator: None })
        );
        for (key, operator) in [
            ("d", Operator::Delete),
            ("c", Operator::Change),
            ("y", Operator::Yank),
        ] {
            assert_eq!(
                parse(&[key, "%"]),
                Some(Command::MatchBracket {
                    operator: Some(operator)
                })
            );
        }
        assert_eq!(parse(&["5", "0", "%"]), None);
        assert_eq!(parse(&["d", "2", "%"]), None);
        assert_eq!(
            parse(&["f", "%"]),
            Some(Command::Find {
                find: CharacterFind {
                    character: '%',
                    backward: false,
                    till: false
                },
                operator: None,
                count: 1
            })
        );
    }

    #[test]
    fn matching_brackets_handle_nesting_strings_and_utf8() {
        let text = "界(a[b]{c})";
        assert_eq!(matching_bracket(text, 0), Some(text.len() - 1));
        assert_eq!(matching_bracket(text, text.len() - 1), Some(3));
        assert_eq!(matching_bracket(text, 5), Some(7));
        assert_eq!(matching_bracket(text, 8), Some(10));
        let text = "(\")\" + '(' + [x])";
        assert_eq!(matching_bracket(text, 0), Some(text.len() - 1));
        assert_eq!(matching_bracket(text, text.len() - 1), Some(0));
        let text = "\"(x)\" (z)";
        assert_eq!(matching_bracket(text, 1), Some(3));
        assert_eq!(matching_bracket(text, 3), Some(1));
        assert_eq!(matching_bracket("plain\n(x)", 0), None);
        assert_eq!(matching_bracket("(missing", 0), None);
        assert_eq!(matching_bracket("missing)", 7), None);
        assert_eq!(matching_bracket("", 0), None);
    }

    #[test]
    fn matching_bracket_edits_include_both_ends_and_the_original_caret() {
        let text = "abc(foo) tail";
        assert_eq!(
            resolve_range(text, 0, Operator::Delete, Target::MatchingBracket, 1),
            Some((0..8, false))
        );
        assert_eq!(
            resolve_range(text, 7, Operator::Delete, Target::MatchingBracket, 1),
            Some((3..8, false))
        );
        assert_eq!(
            resolve_range("no pair", 0, Operator::Change, Target::MatchingBracket, 1),
            None
        );
    }

    #[test]
    fn indentation_counts_rows_and_visual_counts_levels() {
        for (key, kind) in [
            (">", Indentation::Indent),
            ("<", Indentation::Outdent),
            ("=", Indentation::Auto),
        ] {
            assert_eq!(parse(&[key, key]), Some(Command::Indent { kind, count: 1 }));
            assert_eq!(
                parse(&["2", key, "3", key]),
                Some(Command::Indent { kind, count: 6 })
            );
            let mut state = InputState::default();
            state.feed_visual("3");
            assert_eq!(
                state.feed_visual(key),
                Some(Command::Indent { kind, count: 3 })
            );
            assert!(!state.is_pending());
        }
        assert_eq!(parse(&[">", "<"]), None);
        assert_eq!(parse(&["d", ">"]), None);
        let mut state = InputState::default();
        state.feed(">");
        state.feed("escape");
        assert!(!state.is_pending());
        assert_eq!(
            parse(&["f", ">"]),
            Some(Command::Find {
                find: CharacterFind {
                    character: '>',
                    backward: false,
                    till: false
                },
                operator: None,
                count: 1
            })
        );
    }

    #[test]
    fn character_find_parses_literal_targets_and_operator_counts() {
        for (prefix, backward, till) in [
            ("f", false, false),
            ("F", true, false),
            ("t", false, true),
            ("T", true, true),
        ] {
            for character in ['0', 'g', 'u', ' ', ',', '界'] {
                let literal = character.to_string();
                assert_eq!(
                    parse(&["2", "d", "3", prefix, &literal]),
                    Some(Command::Find {
                        find: CharacterFind {
                            character,
                            backward,
                            till
                        },
                        operator: Some(Operator::Delete),
                        count: 6,
                    })
                );
            }
        }
        assert_eq!(
            parse(&["c", "2", ","]),
            Some(Command::RepeatFind {
                reverse: true,
                operator: Some(Operator::Change),
                count: 2
            })
        );
        let mut state = InputState::default();
        state.feed("f");
        assert!(state.waiting_for_character());
        assert_eq!(state.feed("escape"), None);
        assert!(!state.is_pending());
        assert_eq!(parse(&["f", "\n"]), None);
    }

    #[test]
    fn character_find_is_line_bound_and_utf8_safe() {
        let text = "a界b界c\nx界";
        let forward = CharacterFind {
            character: '界',
            backward: false,
            till: false,
        };
        assert_eq!(find_character(text, 0, forward, 2, false), Some(5));
        assert_eq!(find_character(text, 0, forward, 3, false), None);
        assert_eq!(find_character(text, 5, forward, 1, false), None);
        let backward = CharacterFind {
            backward: true,
            ..forward
        };
        assert_eq!(find_character(text, 8, backward, 2, false), Some(1));
        assert_eq!(find_character(text, 10, backward, 1, false), None);
        let till = CharacterFind {
            till: true,
            ..forward
        };
        assert_eq!(find_character(text, 0, till, 1, false), Some(0));
        assert_eq!(find_character(text, 0, till, 1, true), Some(4));
        assert_eq!(find_character(text, 0, till, 2, true), Some(4));
        let backward_till = CharacterFind {
            backward: true,
            till: true,
            ..forward
        };
        assert_eq!(find_character(text, 8, backward_till, 1, true), Some(4));
        assert_eq!(find_character(text, 0, forward, 0, false), None);
    }

    #[test]
    fn character_find_operator_ranges_include_forward_and_exclude_backward_caret() {
        let text = "ab,cd,ef";
        let find = CharacterFind {
            character: ',',
            backward: false,
            till: false,
        };
        assert_eq!(
            resolve_range(
                text,
                0,
                Operator::Delete,
                Target::Find {
                    find,
                    repeated: false
                },
                1
            ),
            Some((0..3, false))
        );
        assert_eq!(
            resolve_range(
                text,
                0,
                Operator::Delete,
                Target::Find {
                    find: CharacterFind { till: true, ..find },
                    repeated: false
                },
                1
            ),
            Some((0..2, false))
        );
        assert_eq!(
            resolve_range(
                text,
                7,
                Operator::Delete,
                Target::Find {
                    find: CharacterFind {
                        backward: true,
                        ..find
                    },
                    repeated: false
                },
                1
            ),
            Some((5..7, false))
        );
        assert_eq!(
            resolve_range(
                text,
                7,
                Operator::Change,
                Target::Find {
                    find: CharacterFind {
                        backward: true,
                        till: true,
                        ..find
                    },
                    repeated: false
                },
                1
            ),
            Some((6..7, false))
        );
        assert_eq!(
            resolve_range(
                text,
                0,
                Operator::Change,
                Target::Find {
                    find,
                    repeated: false
                },
                3
            ),
            None
        );
    }

    #[test]
    fn repeat_preserves_an_unspecified_count() {
        assert_eq!(parse(&["."]), Some(Command::Repeat { count: None }));
        assert_eq!(parse(&["3", "."]), Some(Command::Repeat { count: Some(3) }));
        assert_eq!(parse(&["d", "."]), None);
    }
    #[test]
    fn operator_counts_multiply_and_state_resets() {
        assert_eq!(
            parse(&["2", "d", "3", "w"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Motion(Motion::Word),
                count: 6
            })
        );
        assert_eq!(parse(&["d", "1", "w"]), parse(&["d", "w"]));
        assert_eq!(
            parse(&["d", "2", "a", "p"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Paragraph { around: true },
                count: 2
            })
        );
        assert_eq!(
            parse(&["3", "P"]),
            Some(Command::Paste {
                before: true,
                count: 3
            })
        );
        let mut state = InputState::default();
        state.feed("d");
        state.feed("q");
        assert!(!state.is_pending());
        assert_eq!(
            state.feed("w"),
            Some(Command::Move {
                motion: Motion::Word,
                count: 1
            })
        );
    }
    #[test]
    fn change_word_preserves_following_space() {
        let text = "hello world!";
        assert_eq!(
            edit_range(text, 1, Operator::Delete, Target::Motion(Motion::Word), 1).0,
            1..6
        );
        assert_eq!(
            edit_range(text, 1, Operator::Change, Target::Motion(Motion::Word), 1).0,
            1..5
        );
        assert_eq!(
            edit_range(text, 4, Operator::Change, Target::Motion(Motion::Word), 1).0,
            4..5
        );
        assert_eq!(
            edit_range(text, 1, Operator::Change, Target::Word { around: true }, 1).0,
            0..6
        );
        assert_eq!(
            edit_range(text, 9, Operator::Yank, Target::Word { around: false }, 1).0,
            6..11
        );
        assert_eq!(
            edit_range(
                text,
                11,
                Operator::Delete,
                Target::Word { around: false },
                1
            )
            .0,
            11..12
        );
    }
    #[test]
    fn paragraphs_include_separator_only_when_requested() {
        let text = "one\ntwo\n\nthree\n\nfour";
        assert_eq!(
            edit_range(
                text,
                2,
                Operator::Delete,
                Target::Paragraph { around: false },
                1
            ),
            (0..8, true)
        );
        assert_eq!(
            edit_range(
                text,
                2,
                Operator::Delete,
                Target::Paragraph { around: true },
                1
            ),
            (0..9, true)
        );
        assert_eq!(
            edit_range(
                text,
                2,
                Operator::Delete,
                Target::Paragraph { around: true },
                2
            ),
            (0..16, true)
        );
        assert_eq!(motion(text, 0, Motion::ParagraphNext, 1), 8);
        assert_eq!(motion(text, 10, Motion::ParagraphPrevious, 1), 8);
    }
    #[test]
    fn word_ranges_remain_utf8_boundaries() {
        let text = "你好 世界🙂";
        assert_eq!(
            edit_range(text, 3, Operator::Change, Target::Word { around: true }, 1).0,
            0..7
        );
        assert_eq!(motion(text, 0, Motion::Word, 1), 7);
        assert_eq!(motion(text, 7, Motion::WordEnd, 1), 10);
        assert_eq!(motion(text, 7, Motion::Down, 1), 7);
        assert_eq!(normal_offset("你好\n", 6), 3);
        assert_eq!(normal_offset("", 0), 0);
    }

    #[test]
    fn inner_counts_include_whitespace_objects() {
        let text = "one two three";
        assert_eq!(
            edit_range(text, 0, Operator::Yank, Target::Word { around: false }, 2).0,
            0..4
        );
        assert_eq!(
            edit_range(text, 0, Operator::Yank, Target::Word { around: true }, 2).0,
            0..8
        );
        assert_eq!(
            edit_range(text, 3, Operator::Yank, Target::Word { around: true }, 1).0,
            3..7
        );
        let text = "one\ntwo\n\nthree\n\nfour";
        assert_eq!(
            edit_range(
                text,
                0,
                Operator::Yank,
                Target::Paragraph { around: false },
                2
            )
            .0,
            0..9
        );
        assert_eq!(
            edit_range(
                text,
                8,
                Operator::Yank,
                Target::Paragraph { around: true },
                1
            )
            .0,
            8..15
        );
    }

    #[test]
    fn empty_lines_and_invalid_commands_are_safe() {
        assert_eq!(
            edit_range(
                "\nword",
                0,
                Operator::Delete,
                Target::Motion(Motion::LineEnd),
                1
            )
            .0,
            0..0
        );
        assert_eq!(
            edit_range("", 0, Operator::Delete, Target::Lines, 2).0,
            0..0
        );
        let mut state = InputState::default();
        for key in ["d", "2", "p"] {
            assert_eq!(state.feed(key), None);
        }
        assert!(!state.is_pending());
        for key in ["9", "9", "9", "9", "9"] {
            state.feed(key);
        }
        assert!(!state.is_pending());
    }
    fn object(
        text: &str,
        offset: usize,
        delimiter: char,
        around: bool,
        count: usize,
    ) -> Option<(String, bool)> {
        resolve_range(
            text,
            offset,
            Operator::Change,
            Target::Delimited { delimiter, around },
            count,
        )
        .map(|(range, linewise)| {
            (
                text.get(range).expect("valid UTF-8 range").to_string(),
                linewise,
            )
        })
    }

    #[test]
    fn quoted_objects_handle_escapes_boundaries_and_whitespace() {
        let text = r#"before "hello \"world\" done" tail"#;
        assert_eq!(
            object(text, text.find("world").expect("word"), '"', false, 1),
            Some((r#"hello \"world\" done"#.into(), false))
        );
        assert_eq!(
            object(r#"x "one" tail"#, 5, '"', true, 1),
            Some(("\"one\" ".into(), false))
        );
        assert_eq!(
            object(r#"x "one""#, 5, '"', true, 1),
            Some((" \"one\"".into(), false))
        );
        assert_eq!(
            object(r#""one""#, 0, '"', false, 1),
            Some(("one".into(), false))
        );
        assert_eq!(
            object(r#""one""#, 4, '"', false, 1),
            Some(("one".into(), false))
        );
        assert_eq!(
            object(r#""one""#, 2, '"', false, 2),
            Some(("\"one\"".into(), false))
        );
        assert_eq!(
            object(r#""a\\" tail"#, 2, '"', false, 1),
            Some((r#"a\\"#.into(), false))
        );
        assert_eq!(
            object("let s = '你好';", 9, '\'', false, 1),
            Some(("你好".into(), false))
        );
    }

    #[test]
    fn quoted_objects_search_on_the_current_line() {
        let text = r#"before "one" after "two" tail"#;
        assert_eq!(object(text, 0, '"', false, 1), Some(("one".into(), false)));
        assert_eq!(
            object(text, 12, '"', false, 1),
            Some((" after ".into(), false))
        );
        assert_eq!(object(text, text.len(), '"', false, 1), None);
        assert_eq!(object("before\n\"one\"", 0, '"', false, 1), None);
        assert_eq!(object("\"unclosed", 3, '"', false, 1), None);
        assert_eq!(
            object("\"\"", 1, '"', false, 1),
            Some((String::new(), false))
        );
    }

    #[test]
    fn bracket_objects_use_the_nearest_pair_and_count_enclosing_pairs() {
        let text = "wrap(first, [inner(a, b)], last)";
        let offset = text.find("a, b").expect("inner arguments");
        assert_eq!(
            object(text, offset, '(', false, 1),
            Some(("a, b".into(), false))
        );
        assert_eq!(
            object(text, offset, '(', false, 2),
            Some(("first, [inner(a, b)], last".into(), false))
        );
        assert_eq!(object(text, offset, '(', false, 3), None);
        assert_eq!(
            object(text, offset, '[', true, 1),
            Some(("[inner(a, b)]".into(), false))
        );
        assert_eq!(
            object("call(\"x)y\")", 7, '(', false, 1),
            Some(("\"x)y\"".into(), false))
        );
        assert_eq!(
            object("(one)", 0, '(', false, 1),
            Some(("one".into(), false))
        );
        assert_eq!(
            object("(one)", 4, '(', false, 1),
            Some(("one".into(), false))
        );
        assert_eq!(
            object("call(x)", 0, '(', false, 1),
            Some(("x".into(), false))
        );
        assert_eq!(object("call(x", 5, '(', false, 1), None);
        assert_eq!(object("()", 1, '(', false, 1), Some((String::new(), false)));
    }

    #[test]
    fn multiline_block_objects_preserve_the_bracket_lines() {
        assert_eq!(
            object("{\n  foo();\n  }", 5, '{', false, 1),
            Some(("  foo();\n".into(), true))
        );
        assert_eq!(
            object("{\n  foo();\n  }", 5, '{', true, 1),
            Some(("{\n  foo();\n  }".into(), false))
        );
        assert_eq!(
            object("{\n}", 2, '{', false, 1),
            Some((String::new(), true))
        );
        assert_eq!(
            parse(&["c", "i", "\""]),
            Some(Command::Edit {
                operator: Operator::Change,
                target: Target::Delimited {
                    delimiter: '"',
                    around: false
                },
                count: 1
            })
        );
        assert_eq!(
            parse(&["d", "a", "]"]),
            Some(Command::Edit {
                operator: Operator::Delete,
                target: Target::Delimited {
                    delimiter: '[',
                    around: true
                },
                count: 1
            })
        );
    }
}
