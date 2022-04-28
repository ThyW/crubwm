use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct FocusStack {
    data: VecDeque<u32>,
}

impl FocusStack {
    pub fn focused_client(&self) -> Option<u32> {
        if let Some(first) = self.data.front() {
            return Some(*first);
        }

        None
    }

    pub fn previously_focused_client(&self) -> Option<u32> {
        if let Some(second) = self.data.get(1) {
            return Some(*second);
        }

        None
    }

    pub fn set_focused_client(&mut self, c: u32) {
        if !self.data.contains(&c) {
            self.data.push_front(c);
        } else {
            let mut index = self
                .data
                .iter()
                .enumerate()
                .find(|&(_, d)| d == &c)
                .unwrap()
                .0;
            while index != 0 {
                self.data.swap(index, index - 1);
                index -= 1
            }
        }
    }

    pub fn remove_client(&mut self, c: u32) {
        let tuple = self.data.iter().enumerate().find(|(_, d)| d == &&c);

        if let Some((index, _)) = tuple {
            let _ = self.data.remove(index);
        }
    }
}
