
#[derive(Clone, PartialEq, Eq)]
enum Item {
    None,
    Val(u16),
}

#[derive(Clone, PartialEq)]
pub struct Stack {
    stack: Vec<Item>,
    sp: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack: vec![Item::None; 16],
            sp: 15,
        }
    }

    pub fn pop(&mut self) -> Result<u16, &str> {
        match self.stack[self.sp] {
            Item::None => Err("Empty stack!"),
            Item::Val(x) => {
                self.stack[self.sp] = Item::None;
                self.sp += 1;
                Ok(x)
            }
        }
    }

    pub fn push(&mut self, val: u16) -> Result <(), &str> {
        match self.sp {
            x if x == 0 => Err("Stack is full!"),
            _ => {
                self.sp -= 1;
                self.stack[self.sp] = Item::Val(val);
                Ok(())
            }
        }
    }

    pub fn size(&self) -> usize {
        (15 - self.sp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_stack_test() {
        let mut dickwad = Stack::new();
        dickwad.push(0x84);
        assert_eq!(0x84, dickwad.pop().unwrap());
    }

    #[test]
    fn second_stack_test() {
        let mut dickwad = Stack::new();
        assert_eq!(0, dickwad.size());
        dickwad.push(0x84);
        assert_eq!(1, dickwad.size());
        dickwad.push(0x84);
        assert_eq!(2, dickwad.size());
    }

    #[test]
    fn third_stack_test() {
        let mut dickwad = Stack::new();
        for i in 0..15 {
            dickwad.push(i);
        }
        assert_eq!(Err("Stack is full!"), dickwad.push(16));
    }

    #[test]
    fn fourth_stack_test() {
        let mut dickwad = Stack::new();
        dickwad.push(0x1);
        dickwad.push(0x2);
        dickwad.push(0x3);

        assert_eq!(0x3, dickwad.pop().unwrap());
        assert_eq!(0x2, dickwad.pop().unwrap());
        assert_eq!(0x1, dickwad.pop().unwrap());
        assert_eq!(Err("Empty stack!"), dickwad.pop());
    }

    #[test]
    #[should_panic]
    fn fifth_stack_test() {
        let mut dickwad = Stack::new();
        for _ in 0..16 {
            let res = dickwad.push(0); 
            match res {
                Ok(_) => {},
                Err(e) => panic!("Done panicked: {}", e),
            }
        }
    }
}