#[macro_use] extern crate array_macro;

use std::rc::Rc;
use std::cell::RefCell;

fn main(){}

const STACK_MAX:usize = 256;
const INITIAL_GC_THRESHOLD:usize = 10;

type RefObject = Rc<RefCell<Object>>;

struct Object {
    marked: bool,
    value: ObjectType,

    next: Option<RefObject>,
}

impl Object {
    fn new(typ: ObjectType) -> RefObject {
        Rc::new(RefCell::new(
            Object {
                marked: false,
                value: typ,
                next: None,
            }
        ))
    }

    pub fn mark(&mut self) {
        if self.marked { return; }

        self.marked = true;

        match self.value {
            ObjectType::Pair {ref head, ref tail} => {
                head.borrow_mut().mark(); 
                tail.borrow_mut().mark();
            },
            _ => {}
        }
    }
}

enum ObjectType {
    Int(i32),
    Pair {
        head: RefObject,
        tail: RefObject
    }
}

struct VM {
    stack: [Option<RefObject>; STACK_MAX],
    stack_size: usize,

    first_object: Option<RefObject>,
    num_objects: usize,
    max_objects: usize,
}

impl VM {
    fn new() -> VM {
        VM {
            stack: array![None; STACK_MAX],
            stack_size: 0,
            first_object: None,
            num_objects: 0,
            max_objects: INITIAL_GC_THRESHOLD,
        }
    }

    fn new_object(&mut self, typ: ObjectType) -> RefObject {
        let obj = Object::new(typ);
        obj.borrow_mut().next = self.first_object.clone();
        self.first_object = Some(obj.clone());

        obj
    }

    fn mark_all(&mut self) {
        self.stack.iter_mut()
            .for_each(|obj| {
                if let Some(o) = obj {
                    o.borrow_mut().mark()
                }
            });
    }

    fn sweep(&mut self) {
        let mut object = self.first_object.clone();
        while object.is_some() {
            let obj = object.unwrap();
            let mut obj_mut = obj.borrow_mut();
            if !obj_mut.marked {
                object = obj_mut.next.clone();
                self.num_objects -= 1;
            } else {
                obj_mut.marked = false;
                object = obj_mut.next.clone();
            }
        }
    }

    fn gc(&mut self) {
        let num_objects = self.num_objects;

        self.mark_all();
        self.sweep();

        self.max_objects = num_objects * 2;
    }

    fn push(&mut self, val:RefObject) {
        assert!(self.stack_size < STACK_MAX, "Stack overflow!");
        if self.num_objects == self.max_objects { self.gc(); }
        self.num_objects += 1;

        self.stack[self.stack_size] = Some(val);
        self.stack_size += 1;
    }

    fn pop(&mut self) -> RefObject {
        assert!(self.stack_size > 0, "Stack underflow!");
        self.stack_size -= 1;
        let obj = self.stack[self.stack_size].clone().unwrap();
        self.stack[self.stack_size] = None;

        obj
    }

    fn push_int(&mut self, val: i32) {
        let obj = self.new_object(ObjectType::Int(val));
        self.push(obj);
    }

    fn push_pair(&mut self) {
        let tail = self.pop();
        let head = self.pop();

        let obj = self.new_object(ObjectType::Pair {
            tail: tail,
            head: head
        });

        self.push(obj);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_size_used() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);

        vm.gc();

        assert_eq!(vm.stack_size, 3);
    }

    #[test]
    fn stack_size_equal_in_use() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);

        vm.gc();

        let length = vm.stack.into_iter()
            .filter(|obj| obj.is_some())
            .collect::<Vec<_>>()
            .len();

        assert_eq!(vm.stack_size, length);
    }

    #[test]
    fn stack_collected() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);

        vm.pop();
        vm.pop();

        vm.gc();

        assert_eq!(vm.stack_size, 1);
    }

    #[test]
    fn stack_after_collect_is_equal() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);

        vm.pop();
        vm.pop();

        vm.gc();

        let length = vm.stack.into_iter()
            .filter(|obj| obj.is_some())
            .collect::<Vec<_>>()
            .len();

        assert_eq!(vm.stack_size, length);
    }

    #[test]
    fn stack_nested() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);
        vm.push_int(40);

        vm.push_pair();
        vm.push_pair();
        vm.push_pair();

        vm.gc();

        assert_eq!(vm.stack_size, 1);
    }

    #[test]
    fn stack_nested_count() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);
        vm.push_int(40);

        vm.push_pair();
        vm.push_pair();
        vm.push_pair();

        vm.gc();

        assert_eq!(vm.num_objects, 7);
    }

    #[test]
    fn gc_clean() {
        let mut vm = VM::new();
        vm.push_int(10);
        vm.push_int(20);
        vm.push_int(30);
        vm.push_int(40);

        vm.push_pair();
        vm.push_pair();
        vm.push_pair();

        vm.pop();

        vm.gc();

        assert_eq!(vm.num_objects == 0 && vm.num_objects == vm.stack_size, true);
    }
}