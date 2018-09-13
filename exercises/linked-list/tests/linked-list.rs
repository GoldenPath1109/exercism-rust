extern crate linked_list;
use linked_list::*;

// Add your tests here
#[test]
fn empty_list() {
    let list: LinkedList<i32> = LinkedList::new();
    assert_eq!(list.len(), 0);
}

#[test]
fn single_element() {
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push_back(5);
    assert_eq!(list.len(), 1);
    assert_eq!(list.pop_front(), Some(5));
}

#[test]
fn push_pop_back_multiple() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_back(i);
    }
    assert_eq!(list.len(), 10);

    for i in (0..10).rev() {
        assert_eq!(i, list.pop_back().unwrap());
    }
    assert_eq!(list.len(), 0);
}

#[test]
fn push_pop_front_multiple() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_front(i);
    }
    assert_eq!(list.len(), 10);

    for i in (0..10).rev() {
        assert_eq!(i, list.pop_front().unwrap());
    }
    assert_eq!(list.len(), 0);
}

#[test]
fn push_front_pop_back() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_front(i);
    }
    for i in 0..10 {
        assert_eq!(i, list.pop_back().unwrap());
    }
}

#[test]
fn many_list() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for num in 0..10 {
        list.push_back(num);
    }
    assert_eq!(list.len(), 10);

    for (num, &entered_num) in (0..10).zip(list.iter()) {
        assert_eq!(num, entered_num);
    }
}

// or same number of leaks as double frees
#[test]
fn no_leaks_or_double_frees() {
    use std::cell::Cell;
    struct Counter<'a>(&'a Cell<usize>);

    impl<'a> Drop for Counter<'a> {
        fn drop(&mut self) {
            let num = self.0.get();
            self.0.set(num-1);
        }
    }

    const N: usize  = 15;

    let counter = Cell::new(N);
    let mut list = LinkedList::new();
    for _ in 0..N {
        list.push_back(Counter(&counter));
    }
    assert_eq!(list.len(), N);
    drop(list);
    assert_eq!(counter.get(), 0);
}


#[test]
fn clone_is_equal() {
    let mut list = LinkedList::new();
    for i in 0..10 {
        list.push_back(i);
    }
    let mut list2 = clone_list(&list);

    for _ in 0..10 {
        assert_eq!(list.pop_back(), list2.pop_back());
    }
}

#[test]
fn insert_middle() {
    let mut list = LinkedList::new();
    for i in 0..10 {
        list.push_back(i);
    }
    let mut list2 = clone_list(&list);

    {
        let mut cursor = list.cursor_front();
        for _ in 0..4 {
            cursor.next();
        }
        //cursor.seek_forward(4);
        while let Some(elem) = list2.pop_back() {
            cursor.insert_after(elem);
        }
        //cursor.insert_list_after(&mut list2);
    }

    assert_eq!(list.len(), 20);
    assert_eq!(list2.len(), 0);

    let expected = (0..5).chain(0..10).chain(5..10);

    for (exp, &actual) in expected.zip(list.iter()) {
        assert_eq!(exp, actual);
    }
}

#[test]
fn back_front_changes_on_push_back() {
    let mut backs = vec![];
    let mut fronts = vec![];
    let mut list = LinkedList::new();

    for i in 0..10 {
        list.push_back(i);
        backs.push(list.cursor_back().peek_mut().map_or(std::ptr::null(), |r| r as *const i32));
        fronts.push(list.cursor_front().peek_mut().map_or(std::ptr::null(), |r| r as *const i32));
    }
    fronts.sort();
    fronts.dedup();

    assert_eq!(fronts.len(), 1);
    assert_eq!(fronts[0], backs[0]);

    backs.sort();
    backs.dedup();

    assert_eq!(backs.len(), 10);
}

#[test]
fn back_front_changes_on_push_front() {
    let mut backs = vec![];
    let mut fronts = vec![];
    let mut list = LinkedList::new();

    for i in 0..10 {
        list.push_front(i);
        backs.push(list.cursor_back().peek_mut().map_or(std::ptr::null(), |r| r as *const i32));
        fronts.push(list.cursor_front().peek_mut().map_or(std::ptr::null(), |r| r as *const i32));
    }
    backs.sort();
    backs.dedup();

    assert_eq!(backs.len(), 1);
    assert_eq!(backs[0], fronts[0]);

    fronts.sort();
    fronts.dedup();

    assert_eq!(fronts.len(), 10);
}

#[test]
fn linked_list_is_send_sync() {
    trait AssertSend: Send {}
    trait AssertSync: Sync {}

    impl<T: Send> AssertSend for LinkedList<T> {}
    impl<T: Sync> AssertSync for LinkedList<T> {}
}

#[allow(dead_code)]
#[test]
fn is_covariant() {
    fn a<'a>(x: LinkedList<&'static str>) -> LinkedList<&'a str> {
        x
    }
}

#[test]
fn is_generic() {
    struct Foo;
    LinkedList::<Foo>::new();
}

fn clone_list<T: Clone>(list: &LinkedList<T>) -> LinkedList<T> {
    let mut new_list = LinkedList::new();
    for element in list.iter().cloned() {
        new_list.push_back(element);
    }
    new_list
}

// implement some additional functionality using the required interface
// to limit the amount of code the student has to oversee
trait ListExt<T> {
    fn push_back(&mut self, element: T);
    fn push_front(&mut self, element: T);
    fn pop_back(&mut self) -> Option<T>;
    fn pop_front(&mut self) -> Option<T>;
}

impl<T> ListExt<T> for LinkedList<T> {
    fn push_back(&mut self, element: T) {
        self.cursor_back().insert_after(element);
    }

    fn push_front(&mut self, element: T) {
        self.cursor_front().insert_before(element);
    }

    fn pop_back(&mut self) -> Option<T> {
        self.cursor_back().take()
    }

    fn pop_front(&mut self) -> Option<T> {
        self.cursor_front().take()
    }
}
