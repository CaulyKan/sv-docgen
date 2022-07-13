use std::num::ParseIntError;

pub struct NumberedList {
    list: Vec<u32>,
    sep: String,
}

impl NumberedList {
    pub fn new() -> Self {
        NumberedList {
            list: vec![1],
            sep: ".".to_string(),
        }
    }
    pub fn recall(&self) -> String {
        let s: Vec<String> = self.list.iter().map(|x| x.to_string()).collect();
        s.join(".")
    }
    pub fn step_forward(&mut self) {
        *self.list.last_mut().unwrap() += 1;
    }
    pub fn recall_and_step_forward(&mut self) -> String {
        let result = self.recall();
        self.step_forward();
        result
    }
    pub fn step_forward_and_recall(&mut self) -> String {
        self.step_forward();
        self.recall()
    }
    pub fn go_downstairs(&mut self) {
        self.list.push(1);
    }
    pub fn recall_and_go_downstairs(&mut self) -> String {
        let result = self.recall();
        self.go_downstairs();
        result
    }
    pub fn go_downstairs_and_recall(&mut self) -> String {
        self.go_downstairs();
        self.recall()
    }
    pub fn go_upstairs(&mut self) {
        if self.list.len() > 1 {
            self.list.pop();
        }
    }
    pub fn recall_and_go_upstairs(&mut self) -> String {
        let result = self.recall();
        self.go_upstairs();
        result
    }
    pub fn go_upstairs_and_recall(&mut self) -> String {
        self.go_upstairs();
        self.recall()
    }
    pub fn force(&mut self, new_number: &str) -> Result<(), ParseIntError> {
        let mut parsed = new_number
            .split(self.sep.as_str())
            .map(|x| x.parse::<u32>());
        if let Some(Err(e)) = parsed.find(|x| x.is_err()) {
            Err(e)
        } else {
            self.list = parsed.map(|x| x.unwrap()).collect();
            Ok(())
        }
    }
}
