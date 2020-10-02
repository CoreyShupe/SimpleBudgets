use std::str::FromStr;
use std::borrow::BorrowMut;
use prettytable::{Table, Row, Cell};

pub struct Budget {
    parts: Vec<BudgetPart>,
    expandable_parts: Vec<BudgetPart>,
}

impl Budget {
    pub fn push_part(&mut self, part: BudgetPart) {
        if part.expandable {
            self.expandable_parts.push(part);
        } else {
            self.parts.push(part);
        }
    }

    pub fn print_budget(&self) {
        let mut table = Table::new();
        table.add_row(row!["Name", "Monthly Value", "Expandable"]);

        for x in &self.parts {
            table.add_row(x.row());
        }
        for x in &self.expandable_parts {
            table.add_row(x.row());
        }

        table.printstd();
    }

    pub fn print_budget_requirement(&self, income: f64) {
        let mut table = Table::new();
        table.add_row(row!["Name", "Divided Value"]);

        let mut current_money = income;

        for x in &self.parts {
            if current_money <= 0f64 {
                table.add_row(Row::new(vec![
                    Cell::new(&x.name),
                    Cell::new(&format!("${}", 0))
                ]));
            } else if current_money > x.monthly_value {
                table.add_row(Row::new(vec![
                    Cell::new(&x.name),
                    Cell::new(&format!("${:.2}", x.monthly_value))
                ]));
                current_money -= x.monthly_value;
            } else {
                table.add_row(Row::new(vec![
                    Cell::new(&x.name),
                    Cell::new(&format!("${:.2}", current_money))
                ]));
                current_money = 0f64;
            }
        }

        let mut principles: Vec<BudgetPrinciple> = Vec::<BudgetPrinciple>::new();

        for x in &self.expandable_parts {
            principles.push(x.into())
        }

        let mut size = principles.len();

        while current_money > 0f64 && size > 0 {
            let minimum = BudgetPrinciple::get_min(&principles);

            if current_money / (principles.len() as f64) >= minimum {
                let mut_iterator: &mut [BudgetPrinciple] = principles.borrow_mut();
                for x in mut_iterator {
                    if !x.is_full() {
                        x.add_value(minimum);
                        if x.is_full() {
                            size -= 1;
                        }
                    }
                    current_money -= minimum;
                }
            } else {
                let mut_iterator: &mut [BudgetPrinciple] = principles.borrow_mut();
                for x in mut_iterator {
                    if !x.is_full() {
                        x.add_value(minimum);
                    }
                }
                current_money = 0f64;
            }
        }

        for x in principles {
            table.add_row(Row::new(vec![
                Cell::new(&x.name),
                Cell::new(&format!("${:.2}", x.current_value))
            ]));
        }
        table.add_empty_row();
        table.add_row(Row::new(vec![
            Cell::new("Leftover Money"),
            Cell::new(&format!("${:.2}", current_money))
        ]));
        table.printstd();
    }
}

impl Default for Budget {
    fn default() -> Self {
        Budget { parts: Vec::new(), expandable_parts: Vec::new() }
    }
}

impl Into<String> for Budget {
    fn into(self) -> String {
        let mut value = String::new();

        value += &Budget::budget_vec_to_string(&self.parts);
        value.push('\n');
        value += &Budget::budget_vec_to_string(&self.expandable_parts);

        value
    }
}

impl FromStr for Budget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = Vec::<BudgetPart>::new();
        let mut expandable_parts = Vec::<BudgetPart>::new();

        let mut value_guard = String::new();
        let mut string_iterator = s.chars();

        while let Some(x) = string_iterator.next() {
            match Some(x) {
                Some('\n') => break,
                Some('\t') => {
                    parts.push(value_guard.parse::<BudgetPart>().expect("Failed to parse Budget."));
                    value_guard.clear();
                }
                Some(y) => value_guard.push(y),
                _ => (),
            }
        };

        value_guard.clear();

        while let Some(x) = string_iterator.next() {
            match Some(x) {
                Some('\t') => {
                    expandable_parts.push(value_guard.parse::<BudgetPart>().expect("Failed to parse Budget."));
                    value_guard.clear();
                }
                Some(y) => value_guard.push(y),
                _ => (),
            }
        };

        Ok(Budget { parts, expandable_parts })
    }
}

struct BudgetPrinciple {
    name: String,
    max_value: f64,
    current_value: f64,
}

impl BudgetPrinciple {
    fn get_min(principles: &Vec<BudgetPrinciple>) -> f64 {
        if principles.len() == 0 {
            return 0f64;
        }

        let mut value = std::f64::MAX;

        for x in principles {
            let z = x.max_value - x.current_value;
            if z < value {
                value = z;
            }
        };

        value
    }

    fn add_value(&mut self, value: f64) {
        self.current_value += value;
    }

    fn is_full(&self) -> bool {
        self.current_value >= self.max_value
    }
}

pub struct BudgetPart {
    name: String,
    monthly_value: f64,
    expandable: bool,
}

impl BudgetPart {
    pub fn new(name: String, monthly_value: f64, expandable: bool) -> BudgetPart {
        BudgetPart { name, monthly_value, expandable }
    }

    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&self.name),
            Cell::new(&format!("${:.2}", self.monthly_value)),
            Cell::new(&self.expandable.to_string())
        ])
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Into<String> for &BudgetPart {
    fn into(self) -> String {
        let mut value = String::new();

        value += &self.name.len().to_string();
        value.push('?');
        value += &self.name;
        value += &self.monthly_value.to_string();
        value.push('?');
        value += &self.expandable.to_string();

        value
    }
}

impl Into<BudgetPrinciple> for &BudgetPart {
    fn into(self) -> BudgetPrinciple {
        let max_value = self.monthly_value;
        let name = self.name().clone();
        BudgetPrinciple { name, max_value, current_value: 0f64 }
    }
}

impl FromStr for BudgetPart {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut string_iterator = s.chars();
        let mut value_guard = String::new();
        let mut length = 0usize;

        while let Some(x) = string_iterator.next() {
            if x.is_numeric() {
                value_guard.push(x);
            } else {
                length = value_guard.parse::<usize>().expect("Failed to parse budget part.");
                break;
            }
        };

        value_guard.clear();

        for _ in 0..length {
            value_guard.push(string_iterator.next().expect("Failed to parse budget part."));
        };

        let name = value_guard.clone();

        value_guard.clear();

        while let Some(x) = string_iterator.next() {
            if x.is_numeric() || x == '.' {
                value_guard.push(x);
            } else {
                break;
            }
        };

        let monthly_value = value_guard.parse::<f64>().expect("Failed to parse budget part.");

        value_guard.clear();

        while let Some(x) = string_iterator.next() {
            value_guard.push(x);
        };

        let expandable = value_guard.parse::<bool>().expect("Failed to parse budget part.");

        Ok(BudgetPart { name, monthly_value, expandable })
    }
}

impl Budget {
    fn budget_vec_to_string(parts: &Vec<BudgetPart>) -> String {
        let mut value = String::new();
        for x in parts {
            value += &Into::<String>::into(x);
            value.push('\t');
        };
        value
    }
}